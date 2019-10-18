use futures::{future, Sink, Stream};
use std::convert::TryFrom;
use std::env::args;
use std::process::exit;
use std::str::FromStr;
use tokio::runtime::current_thread::Runtime;
use tokio_xmpp::Component;
use xmpp_parsers::{Jid, Element};
use xmpp_parsers::message::{Body, Message, MessageType};
use xmpp_parsers::presence::{Presence, Show as PresenceShow, Type as PresenceType};

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 3 || args.len() > 5 {
        println!("Usage: {} <jid> <password> [server] [port]", args[0]);
        exit(1);
    }
    let jid = &args[1];
    let password = &args[2];
    let server = &args
        .get(3)
        .unwrap()
        .parse()
        .unwrap_or("127.0.0.1".to_owned());
    let port: u16 = args.get(4).unwrap().parse().unwrap_or(5347u16);

    // tokio_core context
    let mut rt = Runtime::new().unwrap();
    // Component instance
    println!("{} {} {} {}", jid, password, server, port);
    let component = Component::new(jid, password, server, port).unwrap();

    // Make the two interfaces for sending and receiving independent
    // of each other so we can move one into a closure.
    println!("Got it: {}", component.jid.clone());
    let (mut sink, stream) = component.split();
    // Wrap sink in Option so that we can take() it for the send(self)
    // to consume and return it back when ready.
    let mut send = move |stanza| {
        sink.start_send(stanza).expect("start_send");
    };
    // Main loop, processes events
    let done = stream.for_each(|event| {
        if event.is_online() {
            println!("Online!");

            // TODO: replace these hardcoded JIDs
            let presence = make_presence(
                Jid::from_str("test@component.linkmauve.fr/coucou").unwrap(),
                Jid::from_str("linkmauve@linkmauve.fr").unwrap(),
            );
            send(presence);
        } else if let Some(message) = event
            .into_stanza()
            .and_then(|stanza| Message::try_from(stanza).ok())
        {
            // This is a message we'll echo
            match (message.from, message.bodies.get("")) {
                (Some(from), Some(body)) => {
                    if message.type_ != MessageType::Error {
                        let reply = make_reply(from, &body.0);
                        send(reply);
                    }
                }
                _ => (),
            }
        }

        Box::new(future::ok(()))
    });

    // Start polling `done`
    match rt.block_on(done) {
        Ok(_) => (),
        Err(e) => {
            println!("Fatal: {}", e);
            ()
        }
    }
}

// Construct a <presence/>
fn make_presence(from: Jid, to: Jid) -> Element {
    let mut presence = Presence::new(PresenceType::None);
    presence.from = Some(from);
    presence.to = Some(to);
    presence.show = Some(PresenceShow::Chat);
    presence
        .statuses
        .insert(String::from("en"), String::from("Echoing messages."));
    presence.into()
}

// Construct a chat <message/>
fn make_reply(to: Jid, body: &str) -> Element {
    let mut message = Message::new(Some(to));
    message.bodies.insert(String::new(), Body(body.to_owned()));
    message.into()
}
