use std::fmt::Write;

use crossterm::event::KeyCode;
use golem_certificate::{SignedCertificate, schemas::{subject::Subject, certificate::{Certificate, key_usage::{self, KeyUsage}}, validity_period::ValidityPeriod, permissions::Permissions}, Signature, Signer};
use tui::{widgets::{ StatefulWidget }, layout::Rect};
use serde_json::{Value, Map};

use super::{util::{ Component, ComponentStatus, default_style, SizedComponent, Height, Width }, scrollable_text::{ ScrollableText, ScrollableTextState }};

pub struct SignedCertificateDetails {
    render_state: ScrollableTextState,
}

impl SignedCertificateDetails {
    pub fn new(cert: &SignedCertificate) -> Self {
        let text = certifcate_to_text(cert);
        Self { render_state: ScrollableTextState::new(text) }
    }
}

impl Component for SignedCertificateDetails {
    fn render(&mut self, area: Rect, buf: &mut tui::buffer::Buffer) {
        ScrollableText::default()
            .style(default_style())
            .render(area, buf, &mut self.render_state);
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> anyhow::Result<ComponentStatus> {
        let status = match key_event.code {
            KeyCode::Esc => ComponentStatus::Escaped,
            KeyCode::Up => {
                let offset = self.render_state.offset_mut();
                *offset = offset.saturating_sub(1);
                ComponentStatus::Active
            }
            KeyCode::Down => {
                let offset = self.render_state.offset_mut();
                *offset = offset.saturating_add(1);
                ComponentStatus::Active
            }
            _ => ComponentStatus::Active
        };
        Ok(status)
    }
}

impl SizedComponent for SignedCertificateDetails {
    fn get_render_size(&self, area: Rect) -> (Height, Width) {
        ((area.height * 9) / 10, (area.width * 8) / 10)
    }
}


// pub struct SignedCertificate {
//     #[serde(rename = "$schema")]
//     pub schema: String,
//     pub certificate: serde_json::Value,
//     pub signature: Box<Signature<Signer>>,
// }


// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(rename_all = "camelCase")]
// pub struct Certificate {
//     pub validity_period: ValidityPeriod,
//     pub key_usage: KeyUsage,
//     pub permissions: Permissions,
//     pub public_key: Key,
//     pub subject: Subject,
// }


fn certifcate_to_text(cert: &SignedCertificate) -> String {
    let mut buf = String::new();
    let certificate: Certificate = serde_json::from_value(cert.certificate.clone()).unwrap();
    subject_to_buf(&certificate.subject, &mut buf);
    write!(&mut buf, "\n").unwrap();
    validity_period_to_buf(&certificate.validity_period, &mut buf);
    write!(&mut buf, "\n").unwrap();
    permissions_to_buf(&certificate.permissions, &mut buf);
    write!(&mut buf, "\n").unwrap();
    key_usage_to_buf(&certificate.key_usage, &mut buf);
    write!(&mut buf, "\n").unwrap();
    signature_to_buf(&cert.signature, &mut buf);
    buf
}

fn subject_to_buf<W: Write>(subject: &Subject, buf: &mut W) {
    writeln!(buf, "Subject").unwrap();
    writeln!(buf, " Display name: {}", subject.display_name).unwrap();
    writeln!(buf, " Contact").unwrap();
    writeln!(buf, "  Email: {}", subject.contact.email).unwrap();
    for (key, value) in subject.contact.additional_properties.iter() {
        write!(buf, "  {}", key).unwrap();
        if value.is_array() {
            write!(buf, ": ").unwrap();
            array_to_buf(value.as_array().unwrap(), buf, Direction::Horizontal, 2);
        } else if value.is_object() {
            writeln!(buf, "").unwrap();
            object_to_buf(value.as_object().unwrap(), buf, 2);
        } else if value.is_string() {
            writeln!(buf, " {}", value.as_str().unwrap()).unwrap();
        } else {
            writeln!(buf, " {}", serde_json::to_string(value).unwrap()).unwrap();
        }
    }
}

fn validity_period_to_buf<W: Write>(validity_period: &ValidityPeriod, buf: &mut W) {
    writeln!(buf, "Validity Period").unwrap();
    writeln!(buf, " Not before: {}", validity_period.not_before).unwrap();
    writeln!(buf, " Not after:  {}", validity_period.not_after).unwrap();
}


fn permissions_to_buf<W: Write>(permissions: &Permissions, buf: &mut W) {
}

fn key_usage_to_buf<W: Write>(key_usage: &KeyUsage, buf: &mut W) {
}

fn signature_to_buf<W: Write>(signature: &Box<Signature<Signer>>, buf: &mut W) {
    match &signature.signer {
        Signer::SelfSigned => writeln!(buf, "Self signed certificate").unwrap(),
        Signer::Certificate(cert) => {
            writeln!(buf, "Signed by {}", cert.certificate["subject"]["displayName"].as_str().unwrap()).unwrap();
            let signer_text = certifcate_to_text(cert).lines()
                .map(|l| format!(" {}", l)).collect::<Vec<_>>().join("\n");
            write!(buf, "{}", signer_text).unwrap();
        },
    }
}

fn object_to_buf<W: Write>(object: &Map<String, Value>, buf: &mut W, indent: usize) {
    let indentation = vec![" "; indent].join("");
    for (key, value) in object.iter() {
        write!(buf, "{}{}", indentation, key).unwrap();
        if value.is_array() {
            write!(buf, ": ").unwrap();
            array_to_buf(value.as_array().unwrap(), buf, Direction::Horizontal, indent);
        } else if value.is_object() {
            writeln!(buf, "").unwrap();
            object_to_buf(value.as_object().unwrap(), buf, indent + 1);
        } else if value.is_string() {
            writeln!(buf, ": {}", value.as_str().unwrap()).unwrap();
        } else {
            writeln!(buf, ": {}", serde_json::to_string(value).unwrap()).unwrap();
        }
    }
}

enum Direction {
    Horizontal,
    Vertical,
}

fn array_to_buf<W: Write>(array: &Vec<Value>, buf: &mut W, direction: Direction, indent: usize) {
    let (indentation, separator) = match direction {
        Direction::Vertical => (vec![" "; indent].join(""), "\n"),
        Direction::Horizontal => ("".into(), ", "),
    };
    let array_text = array.iter()
        .map(|value| {
            match value.as_str() {
                Some(str) => format!("{}{}", indentation, str),
                None => format!("{}{}", indentation, serde_json::to_string(value).unwrap()),
            }
        })
        .collect::<Vec<_>>()
        .join(separator);
    writeln!(buf, "{}", array_text).unwrap();
}
