// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(missing_docs)]

use try_from::TryFrom;

use minidom::Element;
use jid::Jid;

use error::Error;

use ns;

use iq::{IqGetPayload, IqSetPayload, IqResultPayload};
use data_forms::DataForm;

use pubsub::{NodeName, ItemId, Subscription, SubscriptionId};

// TODO: a better solution would be to split this into a query and a result elements, like for
// XEP-0030.
generate_element!(
    /// A list of affiliations you have on a service, or on a node.
    Affiliations, "affiliations", PUBSUB,
    attributes: [
        /// The optional node name this request pertains to.
        node: Option<NodeName> = "node" => optional,
    ],
    children: [
        /// The actual list of affiliation elements.
        affiliations: Vec<Affiliation> = ("affiliation", PUBSUB) => Affiliation
    ]
);

generate_attribute!(
    /// A list of possible affiliations to a node.
    AffiliationAttribute, "affiliation", {
        /// You are a member of this node, you can subscribe and retrieve items.
        Member => "member",

        /// You don’t have a specific affiliation with this node, you can only subscribe to it.
        None => "none",

        /// You are banned from this node.
        Outcast => "outcast",

        /// You are an owner of this node, and can do anything with it.
        Owner => "owner",

        /// You are a publisher on this node, you can publish and retract items to it.
        Publisher => "publisher",

        /// You can publish and retract items on this node, but not subscribe or retrive items.
        PublishOnly => "publish-only",
    }
);

generate_element!(
    /// An affiliation element.
    Affiliation, "affiliation", PUBSUB,
    attributes: [
        /// The node this affiliation pertains to.
        node: NodeName = "node" => required,

        /// The affiliation you currently have on this node.
        affiliation: AffiliationAttribute = "affiliation" => required,
    ]
);

generate_element!(
    /// Request to configure a new node.
    Configure, "configure", PUBSUB,
    children: [
        /// The form to configure it.
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    ]
);

generate_element!(
    /// Request to create a new node.
    Create, "create", PUBSUB,
    attributes: [
        /// The node name to create, if `None` the service will generate one.
        node: Option<NodeName> = "node" => optional,
    ]
);

generate_element!(
    /// Request for a default node configuration.
    Default, "default", PUBSUB,
    attributes: [
        /// The node targetted by this request, otherwise the entire service.
        node: Option<NodeName> = "node" => optional,

        // TODO: do we really want to support collection nodes?
        // type: String = "type" => optional,
    ]
);

generate_element!(
    /// A request for a list of items.
    Items, "items", PUBSUB,
    attributes: [
        // TODO: should be an xs:positiveInteger, that is, an unbounded int ≥ 1.
        /// Maximum number of items returned.
        max_items: Option<u32> = "max_items" => optional,

        /// The node queried by this request.
        node: NodeName = "node" => required,

        /// The subscription identifier related to this request.
        subid: Option<SubscriptionId> = "subid" => optional,
    ],
    children: [
        /// The actual list of items returned.
        items: Vec<Item> = ("item", PUBSUB) => Item
    ]
);

/// An item from a PubSub node.
#[derive(Debug, Clone)]
pub struct Item {
    /// The payload of this item, in an arbitrary namespace.
    pub payload: Option<Element>,

    /// The 'id' attribute of this item.
    pub id: Option<ItemId>,
}

impl TryFrom<Element> for Item {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Item, Error> {
        check_self!(elem, "item", PUBSUB);
        check_no_unknown_attributes!(elem, "item", ["id"]);
        let mut payloads = elem.children().cloned().collect::<Vec<_>>();
        let payload = payloads.pop();
        if !payloads.is_empty() {
            return Err(Error::ParseError("More than a single payload in item element."));
        }
        Ok(Item {
            payload,
            id: get_attr!(elem, "id", optional),
        })
    }
}

impl From<Item> for Element {
    fn from(item: Item) -> Element {
        Element::builder("item")
                .ns(ns::PUBSUB)
                .attr("id", item.id)
                .append(item.payload)
                .build()
    }
}

generate_element!(
    /// The options associated to a subscription request.
    Options, "options", PUBSUB,
    attributes: [
        /// The JID affected by this request.
        jid: Jid = "jid" => required,

        /// The node affected by this request.
        node: Option<NodeName> = "node" => optional,

        /// The subscription identifier affected by this request.
        subid: Option<SubscriptionId> = "subid" => optional,
    ],
    children: [
        /// The form describing the subscription.
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    ]
);

generate_element!(
    /// Request to publish items to a node.
    Publish, "publish", PUBSUB,
    attributes: [
        /// The target node for this operation.
        node: NodeName = "node" => required,
    ],
    children: [
        /// The items you want to publish.
        items: Vec<Item> = ("item", PUBSUB) => Item
    ]
);

generate_element!(
    /// The options associated to a publish request.
    PublishOptions, "publish-options", PUBSUB,
    children: [
        /// The form describing these options.
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    ]
);

generate_attribute!(
    /// Whether a retract request should notify subscribers or not.
    Notify, "notify", bool
);

generate_element!(
    /// A request to retract some items from a node.
    Retract, "retract", PUBSUB,
    attributes: [
        /// The node affected by this request.
        node: NodeName = "node" => required,

        /// Whether a retract request should notify subscribers or not.
        notify: Notify = "notify" => default,
    ],
    children: [
        /// The items affected by this request.
        items: Vec<Item> = ("item", PUBSUB) => Item
    ]
);

/// Indicate that the subscription can be configured.
#[derive(Debug, Clone)]
pub struct SubscribeOptions {
    /// If `true`, the configuration is actually required.
    required: bool,
}

impl TryFrom<Element> for SubscribeOptions {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Self, Error> {
        check_self!(elem, "subscribe-options", PUBSUB);
        check_no_attributes!(elem, "subscribe-options");
        let mut required = false;
        for child in elem.children() {
            if child.is("required", ns::PUBSUB) {
                if required {
                    return Err(Error::ParseError("More than one required element in subscribe-options."));
                }
                required = true;
            } else {
                return Err(Error::ParseError("Unknown child in subscribe-options element."));
            }
        }
        Ok(SubscribeOptions { required })
    }
}

impl From<SubscribeOptions> for Element {
    fn from(subscribe_options: SubscribeOptions) -> Element {
        Element::builder("subscribe-options")
            .ns(ns::PUBSUB)
            .append(if subscribe_options.required {
                 vec!(Element::builder("required")
                     .ns(ns::PUBSUB)
                     .build())
             } else {
                 vec!()
             })
            .build()
    }
}

generate_element!(
    /// A request to subscribe a JID to a node.
    Subscribe, "subscribe", PUBSUB,
    attributes: [
        /// The JID being subscribed.
        jid: Jid = "jid" => required,

        /// The node to subscribe to.
        node: Option<NodeName> = "node" => optional,
    ]
);

generate_element!(
    /// A request for current subscriptions.
    Subscriptions, "subscriptions", PUBSUB,
    attributes: [
        /// The node to query.
        node: Option<NodeName> = "node" => optional,
    ],
    children: [
        /// The list of subscription elements returned.
        subscription: Vec<SubscriptionElem> = ("subscription", PUBSUB) => SubscriptionElem
    ]
);

generate_element!(
    /// A subscription element, describing the state of a subscription.
    SubscriptionElem, "subscription", PUBSUB,
    attributes: [
        /// The JID affected by this subscription.
        jid: Jid = "jid" => required,

        /// The node affected by this subscription.
        node: Option<NodeName> = "node" => optional,

        /// The subscription identifier for this subscription.
        subid: Option<SubscriptionId> = "subid" => optional,

        /// The state of the subscription.
        subscription: Option<Subscription> = "subscription" => optional,
    ],
    children: [
        /// The options related to this subscription.
        subscribe_options: Option<SubscribeOptions> = ("subscribe-options", PUBSUB) => SubscribeOptions
    ]
);

generate_element!(
    /// An unsubscribe request.
    Unsubscribe, "unsubscribe", PUBSUB,
    attributes: [
        /// The JID affected by this request.
        jid: Jid = "jid" => required,

        /// The node affected by this request.
        node: Option<NodeName> = "node" => optional,

        /// The subscription identifier for this subscription.
        subid: Option<SubscriptionId> = "subid" => optional,
    ]
);

/// Main payload used to communicate with a PubSub service.
///
/// `<pubsub xmlns="http://jabber.org/protocol/pubsub"/>`
#[derive(Debug, Clone)]
pub enum PubSub {
    /// Request to create a new node, with optional suggested name and suggested configuration.
    Create {
        /// The create request.
        create: Create,

        /// The configure request for the new node.
        configure: Option<Configure>
    },

    /// Request to publish items to a node, with optional options.
    Publish {
        /// The publish request.
        publish: Publish,

        /// The options related to this publish request.
        publish_options: Option<PublishOptions>
    },

    /// A list of affiliations you have on a service, or on a node.
    Affiliations(Affiliations),

    /// Request for a default node configuration.
    Default(Default),

    /// A request for a list of items.
    Items(Items),

    /// A request to retract some items from a node.
    Retract(Retract),

    /// A request about a subscription.
    Subscription(SubscriptionElem),

    /// A request for current subscriptions.
    Subscriptions(Subscriptions),

    /// An unsubscribe request.
    Unsubscribe(Unsubscribe),
}

impl IqGetPayload for PubSub {}
impl IqSetPayload for PubSub {}
impl IqResultPayload for PubSub {}

impl TryFrom<Element> for PubSub {
    type Err = Error;

    fn try_from(elem: Element) -> Result<PubSub, Error> {
        check_self!(elem, "pubsub", PUBSUB);
        check_no_attributes!(elem, "pubsub");

        let mut payload = None;
        for child in elem.children() {
            if child.is("create", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let create = Create::try_from(child.clone())?;
                payload = Some(PubSub::Create { create, configure: None });
            } else if child.is("configure", ns::PUBSUB) {
                if let Some(PubSub::Create { create, configure }) = payload {
                    if configure.is_some() {
                        return Err(Error::ParseError("Configure is already defined in pubsub element."));
                    }
                    let configure = Some(Configure::try_from(child.clone())?);
                    payload = Some(PubSub::Create { create, configure });
                } else {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
            } else if child.is("publish", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let publish = Publish::try_from(child.clone())?;
                payload = Some(PubSub::Publish { publish, publish_options: None });
            } else if child.is("publish-options", ns::PUBSUB) {
                if let Some(PubSub::Publish { publish, publish_options }) = payload {
                    if publish_options.is_some() {
                        return Err(Error::ParseError("Publish-options are already defined in pubsub element."));
                    }
                    let publish_options = Some(PublishOptions::try_from(child.clone())?);
                    payload = Some(PubSub::Publish { publish, publish_options });
                } else {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
            } else if child.is("affiliations", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let affiliations = Affiliations::try_from(child.clone())?;
                payload = Some(PubSub::Affiliations(affiliations));
            } else if child.is("default", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let default = Default::try_from(child.clone())?;
                payload = Some(PubSub::Default(default));
            } else if child.is("items", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let items = Items::try_from(child.clone())?;
                payload = Some(PubSub::Items(items));
            } else if child.is("retract", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let retract = Retract::try_from(child.clone())?;
                payload = Some(PubSub::Retract(retract));
            } else if child.is("subscription", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let subscription = SubscriptionElem::try_from(child.clone())?;
                payload = Some(PubSub::Subscription(subscription));
            } else if child.is("subscriptions", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let subscriptions = Subscriptions::try_from(child.clone())?;
                payload = Some(PubSub::Subscriptions(subscriptions));
            } else if child.is("unsubscribe", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let unsubscribe = Unsubscribe::try_from(child.clone())?;
                payload = Some(PubSub::Unsubscribe(unsubscribe));
            } else {
                return Err(Error::ParseError("Unknown child in pubsub element."));
            }
        }
        Ok(payload.ok_or(Error::ParseError("No payload in pubsub element."))?)
    }
}

impl From<PubSub> for Element {
    fn from(pubsub: PubSub) -> Element {
        Element::builder("pubsub")
            .ns(ns::PUBSUB)
            .append(match pubsub {
                 PubSub::Create { create, configure } => {
                     let mut elems = vec!(Element::from(create));
                     if let Some(configure) = configure {
                         elems.push(Element::from(configure));
                     }
                     elems
                 },
                 PubSub::Publish { publish, publish_options } => {
                     let mut elems = vec!(Element::from(publish));
                     if let Some(publish_options) = publish_options {
                         elems.push(Element::from(publish_options));
                     }
                     elems
                 },
                 PubSub::Affiliations(affiliations) => vec!(Element::from(affiliations)),
                 PubSub::Default(default) => vec!(Element::from(default)),
                 PubSub::Items(items) => vec!(Element::from(items)),
                 PubSub::Retract(retract) => vec!(Element::from(retract)),
                 PubSub::Subscription(subscription) => vec!(Element::from(subscription)),
                 PubSub::Subscriptions(subscriptions) => vec!(Element::from(subscriptions)),
                 PubSub::Unsubscribe(unsubscribe) => vec!(Element::from(unsubscribe)),
             })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compare_elements::NamespaceAwareCompare;

    #[test]
    fn create() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><create/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Create { create, configure } => {
                assert!(create.node.is_none());
                assert!(configure.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert!(elem1.compare_to(&elem2));

        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><create node='coucou'/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Create { create, configure } => {
                assert_eq!(&create.node.unwrap().0, "coucou");
                assert!(configure.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn create_and_configure() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><create/><configure/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Create { create, configure } => {
                assert!(create.node.is_none());
                assert!(configure.unwrap().form.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn publish() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><publish node='coucou'/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Publish { publish, publish_options } => {
                assert_eq!(&publish.node.0, "coucou");
                assert!(publish_options.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn publish_with_publish_options() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><publish node='coucou'/><publish-options/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Publish { publish, publish_options } => {
                assert_eq!(&publish.node.0, "coucou");
                assert!(publish_options.unwrap().form.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn invalid_empty_pubsub() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'/>".parse().unwrap();
        let error = PubSub::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "No payload in pubsub element.");
    }

    #[test]
    fn publish_option() {
        let elem: Element = "<publish-options xmlns='http://jabber.org/protocol/pubsub'><x xmlns='jabber:x:data' type='submit'><field var='FORM_TYPE' type='hidden'><value>http://jabber.org/protocol/pubsub#publish-options</value></field></x></publish-options>".parse().unwrap();
        let publish_options = PublishOptions::try_from(elem).unwrap();
        assert_eq!(&publish_options.form.unwrap().form_type.unwrap(), "http://jabber.org/protocol/pubsub#publish-options");
    }

    #[test]
    fn subscribe_options() {
        let elem1: Element = "<subscribe-options xmlns='http://jabber.org/protocol/pubsub'/>".parse().unwrap();
        let subscribe_options1 = SubscribeOptions::try_from(elem1).unwrap();
        assert_eq!(subscribe_options1.required, false);

        let elem2: Element = "<subscribe-options xmlns='http://jabber.org/protocol/pubsub'><required/></subscribe-options>".parse().unwrap();
        let subscribe_options2 = SubscribeOptions::try_from(elem2).unwrap();
        assert_eq!(subscribe_options2.required, true);
    }
}
