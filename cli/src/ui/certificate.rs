use std::fmt::Write;

use crossterm::event::KeyCode;
use golem_certificate::{SignedCertificate, schemas::{subject::Subject, certificate::{Certificate, key_usage::KeyUsage}, validity_period::ValidityPeriod, permissions::{Permissions, OutboundPermissions::{Unrestricted, Urls}}}, Signature, Signer};
use tui::{widgets::{ StatefulWidget }, layout::Rect};
use serde_json::{Value, Map};

use super::{util::{ Component, ComponentStatus, default_style, SizedComponent, Height, Width }, scrollable_text::{ ScrollableText, ScrollableTextState }};

pub struct SignedCertificateDetails {
    render_state: ScrollableTextState,
}

impl SignedCertificateDetails {
    pub fn new(cert: &SignedCertificate, indent: usize, detailed_signer: bool) -> Self {
        let text = certificate_to_string(cert, indent, detailed_signer);
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

struct StringBuffer {
    buf: String,
    indent: String,
    indent_level: usize,
}

impl StringBuffer {
    fn new(indent: usize) -> Self {
        let buf = String::new();
        let indent = vec![" "; indent].join("");
        let indent_level = 0;
        Self { buf, indent, indent_level }
    }

    fn increase_indent_level(&mut self) {
        self.indent_level += 1;
    }

    fn decrease_indent_level(&mut self) {
        self.indent_level -= 1;
    }

    fn buf_mut_with_indent(&mut self) -> &mut String {
        (0..self.indent_level)
            .for_each(|_| write!(&mut self.buf, "{}", self.indent).unwrap());
        &mut self.buf
    }

    fn buf_mut(&mut self) -> &mut String {
        &mut self.buf
    }

    fn add_empty_line(&mut self) {
        writeln!(&mut self.buf, "").unwrap();
    }

    fn into_inner(self) -> String {
        self.buf
    }
}


fn certificate_to_string(cert: &SignedCertificate, indent: usize, detailed_signer: bool) -> String {
    let mut buf = StringBuffer::new(indent);
    write_certificate(&mut buf, cert, detailed_signer);
    buf.into_inner()
}

fn write_certificate(buf: &mut StringBuffer, cert: &SignedCertificate, detailed_signer: bool) {
    let certificate: Certificate = serde_json::from_value(cert.certificate.clone()).unwrap();
    write_subject(buf, &certificate.subject);
    buf.add_empty_line();
    validity_write_period(buf, &certificate.validity_period);
    buf.add_empty_line();
    write_permissions(buf, &certificate.permissions);
    buf.add_empty_line();
    write_key_usage(buf, &certificate.key_usage);
    buf.add_empty_line();
    write_signature(buf, &cert.signature, detailed_signer);
}

fn write_subject(buf: &mut StringBuffer, subject: &Subject) {
    writeln!(buf.buf_mut_with_indent(), "Subject").unwrap();
    buf.increase_indent_level();
    writeln!(buf.buf_mut_with_indent(), "Display name: {}", subject.display_name).unwrap();
    writeln!(buf.buf_mut_with_indent(), "Contact").unwrap();
    buf.increase_indent_level();
    writeln!(buf.buf_mut_with_indent(), "Email: {}", subject.contact.email).unwrap();
    for (key, value) in subject.contact.additional_properties.iter() {
        write!(buf.buf_mut_with_indent(), "{}", key).unwrap();
        if value.is_array() {
            write!(buf.buf_mut(), ": ").unwrap();
            write_value_as_array(buf, value, Direction::Horizontal);
        } else if value.is_object() {
            writeln!(buf.buf_mut(), "").unwrap();
            buf.increase_indent_level();
            write_object(value.as_object().unwrap(), buf);
            buf.decrease_indent_level();
        } else if value.is_string() {
            writeln!(buf.buf_mut(), " {}", value.as_str().unwrap()).unwrap();
        } else {
            writeln!(buf.buf_mut(), " {}", serde_json::to_string(value).unwrap()).unwrap();
        }
    }
    buf.decrease_indent_level();
    for (key, value) in subject.additional_properties.iter() {
        write!(buf.buf_mut_with_indent(), "{}", key).unwrap();
        if value.is_array() {
            write!(buf.buf_mut(), ": ").unwrap();
            write_value_as_array(buf, value, Direction::Horizontal);
        } else if value.is_object() {
            writeln!(buf.buf_mut(), "").unwrap();
            buf.increase_indent_level();
            write_object(value.as_object().unwrap(), buf);
            buf.decrease_indent_level();
        } else if value.is_string() {
            writeln!(buf.buf_mut(), " {}", value.as_str().unwrap()).unwrap();
        } else {
            writeln!(buf.buf_mut(), " {}", serde_json::to_string(value).unwrap()).unwrap();
        }
    }
    buf.decrease_indent_level();
}

fn validity_write_period(buf: &mut StringBuffer, validity_period: &ValidityPeriod) {
    writeln!(buf.buf_mut_with_indent(), "Validity Period").unwrap();
    buf.increase_indent_level();
    writeln!(buf.buf_mut_with_indent(), "Not before: {}", validity_period.not_before).unwrap();
    writeln!(buf.buf_mut_with_indent(), "Not after:  {}", validity_period.not_after).unwrap();
    buf.decrease_indent_level();
}

fn write_permissions(buf: &mut StringBuffer, permissions: &Permissions) {
    write!(buf.buf_mut_with_indent(), "Permissions").unwrap();
    match permissions {
        Permissions::All => writeln!(buf.buf_mut(), ": All").unwrap(),
        Permissions::Object(details) => {
            if let Some(outbound) = &details.outbound {
                writeln!(buf.buf_mut(), "").unwrap();
                buf.increase_indent_level();
                write!(buf.buf_mut_with_indent(), "Outbound").unwrap();
                match outbound {
                    Unrestricted => writeln!(buf.buf_mut(), ": Unrestricted").unwrap(),
                    Urls(urls) => {
                        let mut array = urls.iter()
                            .map(|url| url.clone().into())
                            .collect::<Vec<String>>();
                        array.sort();
                        writeln!(buf.buf_mut(), "").unwrap();
                        buf.increase_indent_level();
                        write_array(buf, &array, Direction::Vertical);
                        buf.decrease_indent_level();
                    },
                }
                buf.decrease_indent_level();
            } else {
                writeln!(buf.buf_mut(), ": None").unwrap();
            }
        }
    }
}

fn write_key_usage(buf: &mut StringBuffer, key_usage: &KeyUsage) {
    write!(buf.buf_mut_with_indent(), "Key usage: ").unwrap();
    match key_usage {
        KeyUsage::All => writeln!(buf.buf_mut(), "All").unwrap(),
        KeyUsage::Limited(usage) => {
            let mut key_usage = usage.iter()
                .map(|u| {
                    let value = serde_json::to_value(u).unwrap();
                    value.as_str().unwrap().to_owned()
                })
                .collect::<Vec<_>>();
            key_usage.sort();
            write_array(buf, &key_usage, Direction::Horizontal);
        },
    }
}

fn write_signature(buf: &mut StringBuffer, signature: &Box<Signature<Signer>>, detailed_signer: bool) {
    match &signature.signer {
        Signer::SelfSigned => writeln!(buf.buf_mut_with_indent(), "Self signed certificate").unwrap(),
        Signer::Certificate(cert) => {
            writeln!(buf.buf_mut_with_indent(), "Signed by {}", cert.certificate["subject"]["displayName"].as_str().unwrap()).unwrap();
            if detailed_signer {
                buf.increase_indent_level();
                write_certificate(buf, cert, detailed_signer);
                buf.decrease_indent_level();
            }
        },
    }
}

fn write_object(object: &Map<String, Value>, buf: &mut StringBuffer) {
    for (key, value) in object.iter() {
        write!(buf.buf_mut_with_indent(), "{}", key).unwrap();
        if value.is_array() {
            write!(buf.buf_mut(), ":").unwrap();
            write_value_as_array(buf, value, Direction::Horizontal);
        } else if value.is_object() {
            writeln!(buf.buf_mut(), "").unwrap();
            buf.increase_indent_level();
            write_object(value.as_object().unwrap(), buf);
            buf.decrease_indent_level();
        } else if value.is_string() {
            writeln!(buf.buf_mut(), ": {}", value.as_str().unwrap()).unwrap();
        } else {
            writeln!(buf.buf_mut(), ": {}", serde_json::to_string(value).unwrap()).unwrap();
        }
    }
}

enum Direction {
    Horizontal,
    Vertical,
}

fn write_value_as_array(buf: &mut StringBuffer, value: &Value, direction: Direction) {
    let values = value.as_array().unwrap()
        .iter()
        .map(|v| match v.as_str() {
            Some(str) => str.to_string(),
            None => serde_json::to_string(v).unwrap(),
        })
        .collect::<Vec<_>>();
    write_array(buf, &values, direction)
}

fn write_array(buf: &mut StringBuffer, array: &Vec<String>, direction: Direction) {
    array.iter()
        .for_each(|element| {
            match direction {
                Direction::Horizontal => write!(buf.buf_mut(), " {},", element).unwrap(),
                Direction::Vertical => writeln!(buf.buf_mut_with_indent(), "{}", element).unwrap(),
            }
        });
    buf.buf_mut().pop();
    writeln!(buf.buf_mut(), "").unwrap();
}
