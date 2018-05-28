// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use helpers::TrimmedPlainText;

generate_element_with_text!(URI, "uri", MEDIA_ELEMENT,
    [
        type_: String = "type" => required
    ],
    uri: TrimmedPlainText<String>
);

generate_element!(MediaElement, "media", MEDIA_ELEMENT,
    attributes: [
        width: Option<usize> = "width" => optional,
        height: Option<usize> = "height" => optional
    ],
    children: [
        uris: Vec<URI> = ("uri", MEDIA_ELEMENT) => URI
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;
    use minidom::Element;
    use error::Error;
    use data_forms::DataForm;
    use std::error::Error as StdError;

    #[test]
    fn test_simple() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element'/>".parse().unwrap();
        let media = MediaElement::try_from(elem).unwrap();
        assert!(media.width.is_none());
        assert!(media.height.is_none());
        assert!(media.uris.is_empty());
    }

    #[test]
    fn test_width_height() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element' width='32' height='32'/>".parse().unwrap();
        let media = MediaElement::try_from(elem).unwrap();
        assert_eq!(media.width.unwrap(), 32);
        assert_eq!(media.height.unwrap(), 32);
    }

    #[test]
    fn test_uri() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element'><uri type='text/html'>https://example.org/</uri></media>".parse().unwrap();
        let media = MediaElement::try_from(elem).unwrap();
        assert_eq!(media.uris.len(), 1);
        assert_eq!(media.uris[0].type_, "text/html");
        assert_eq!(media.uris[0].uri, "https://example.org/");
    }

    #[test]
    fn test_invalid_width_height() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element' width=''/>".parse().unwrap();
        let error = MediaElement::try_from(elem).unwrap_err();
        let error = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(error.description(), "cannot parse integer from empty string");

        let elem: Element = "<media xmlns='urn:xmpp:media-element' width='coucou'/>".parse().unwrap();
        let error = MediaElement::try_from(elem).unwrap_err();
        let error = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(error.description(), "invalid digit found in string");

        let elem: Element = "<media xmlns='urn:xmpp:media-element' height=''/>".parse().unwrap();
        let error = MediaElement::try_from(elem).unwrap_err();
        let error = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(error.description(), "cannot parse integer from empty string");

        let elem: Element = "<media xmlns='urn:xmpp:media-element' height='-10'/>".parse().unwrap();
        let error = MediaElement::try_from(elem).unwrap_err();
        let error = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(error.description(), "invalid digit found in string");
    }

    #[test]
    fn test_unknown_child() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element'><coucou/></media>".parse().unwrap();
        let error = MediaElement::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in media element.");
    }

    #[test]
    fn test_bad_uri() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element'><uri>https://example.org/</uri></media>".parse().unwrap();
        let error = MediaElement::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");

        let elem: Element = "<media xmlns='urn:xmpp:media-element'><uri type='text/html'/></media>".parse().unwrap();
        let error = MediaElement::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "URI missing in uri.");
    }

    #[test]
    fn test_xep_ex1() {
        let elem: Element = r#"
<media xmlns='urn:xmpp:media-element'>
  <uri type='audio/x-wav'>
    http://victim.example.com/challenges/speech.wav?F3A6292C
  </uri>
  <uri type='audio/ogg; codecs=speex'>
    cid:sha1+a15a505e360702b79c75a5f67773072ed392f52a@bob.xmpp.org
  </uri>
  <uri type='audio/mpeg'>
    http://victim.example.com/challenges/speech.mp3?F3A6292C
  </uri>
</media>"#.parse().unwrap();
        let media = MediaElement::try_from(elem).unwrap();
        assert!(media.width.is_none());
        assert!(media.height.is_none());
        assert_eq!(media.uris.len(), 3);
        assert_eq!(media.uris[0].type_, "audio/x-wav");
        assert_eq!(media.uris[0].uri, "http://victim.example.com/challenges/speech.wav?F3A6292C");
        assert_eq!(media.uris[1].type_, "audio/ogg; codecs=speex");
        assert_eq!(media.uris[1].uri, "cid:sha1+a15a505e360702b79c75a5f67773072ed392f52a@bob.xmpp.org");
        assert_eq!(media.uris[2].type_, "audio/mpeg");
        assert_eq!(media.uris[2].uri, "http://victim.example.com/challenges/speech.mp3?F3A6292C");
    }

    #[test]
    fn test_xep_ex2() {
        let elem: Element = r#"
<x xmlns='jabber:x:data' type='form'>
  [ ... ]
  <field var='ocr'>
    <media xmlns='urn:xmpp:media-element'
           height='80'
           width='290'>
      <uri type='image/jpeg'>
        http://www.victim.com/challenges/ocr.jpeg?F3A6292C
      </uri>
      <uri type='image/jpeg'>
        cid:sha1+f24030b8d91d233bac14777be5ab531ca3b9f102@bob.xmpp.org
      </uri>
    </media>
  </field>
  [ ... ]
</x>"#.parse().unwrap();
        let form = DataForm::try_from(elem).unwrap();
        assert_eq!(form.fields.len(), 1);
        assert_eq!(form.fields[0].var, "ocr");
        assert_eq!(form.fields[0].media[0].width, Some(290));
        assert_eq!(form.fields[0].media[0].height, Some(80));
        assert_eq!(form.fields[0].media[0].uris[0].type_, "image/jpeg");
        assert_eq!(form.fields[0].media[0].uris[0].uri, "http://www.victim.com/challenges/ocr.jpeg?F3A6292C");
        assert_eq!(form.fields[0].media[0].uris[1].type_, "image/jpeg");
        assert_eq!(form.fields[0].media[0].uris[1].uri, "cid:sha1+f24030b8d91d233bac14777be5ab531ca3b9f102@bob.xmpp.org");
    }
}
