use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use crate::{structs::{Attachment, Chat, Message, Participant}, util::{apple_to_unix, unix_to_apple}};

use image::io::Reader as ImageReader;

pub struct Database {
    conn: Connection,
    last_read_time: u128,
}

#[derive(Serialize)]
pub struct ChatCounts {
    total: usize,
    breakdown: HashMap<String, usize>,
}

impl Database {
    pub fn new(last_read_time: u128) -> Self {
        let database_path = format!("{}/Library/Messages/chat.db", std::env::var("HOME").unwrap());
        println!("opening: {database_path}");
        let conn = Connection::open(database_path).unwrap();
        Self {
            last_read_time,
            conn,
        }
    }

    pub fn poll(&mut self) {
        let mut stmt = self.conn.prepare("SELECT guid FROM message WHERE date > ?").unwrap();
        //apple time starts from 1-1-2001 in nanoseconds 😤
        println!("{}", unix_to_apple(self.last_read_time));
        let guids_iter = stmt.query_map([TryInto::<u64>::try_into(self.last_read_time-978307200000000000).unwrap()], |row| {
            Ok(row.get::<&str, String>("guid")?)
        }).unwrap().filter_map(|guid| {
            if let Ok(guid) = guid {
                return self.get_message_by_guid(guid, true, true);
            }
            None
        });
        self.last_read_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    }

    pub fn get_chat_by_guid(&self, guid: String, last_message: bool, participants: bool) -> Option<Chat> {
        let mut stmt = self.conn.prepare("SELECT * FROM chat WHERE guid = ?").unwrap();
        return stmt.query_row([guid.clone()], |row| { 
            let original_rowid = row.get_unwrap("ROWID");
            let participants = if participants {
                Some(self.get_chat_participants(original_rowid).unwrap())
            } else {
                None
            };
            let last_message = if last_message {
                self.get_last_chat_message(original_rowid)
            } else {
                None
            };
            Ok(
                Chat {
                    original_rowid,
                    guid: guid,
                    participants,
                    last_message,
                    style: row.get_unwrap("style"),
                    chat_identifier: row.get_unwrap("chat_identifier"),
                    is_archived: row.get_unwrap("is_archived"),
                    is_filtered: row.get_unwrap("is_filtered"),
                    display_name: row.get_unwrap("display_name"),
                    group_id: row.get_unwrap("group_id"),
                    last_addressed_handle: row.get_unwrap("last_addressed_handle")
                }
        ) }).optional().unwrap();
    }

    pub fn query_chats(&self, limit: usize, offset: usize, sort: Option<String>, last_message: bool, participants: bool) -> Vec<Chat> {
        let mut stmt = self.conn.prepare("SELECT * FROM chat LIMIT ?").unwrap();
        let mut chats: Vec<Chat> = stmt.query_map([limit+offset], |row| {
            let original_rowid = row.get_unwrap("ROWID");
            let participants = if participants {
                Some(self.get_chat_participants(original_rowid).unwrap())
            } else {
                None
            };
            let last_message = if last_message {
                self.get_last_chat_message(original_rowid)
            } else {
                None
            };
            Ok(
                Chat {
                    original_rowid,
                    guid: row.get_unwrap("guid"),
                    participants,
                    last_message,
                    style: row.get_unwrap("style"),
                    chat_identifier: row.get_unwrap("chat_identifier"),
                    is_archived: row.get_unwrap("is_archived"),
                    is_filtered: row.get_unwrap("is_filtered"),
                    display_name: row.get("display_name").ok(),
                    group_id: row.get_unwrap("group_id"),
                    last_addressed_handle: row.get_unwrap("last_addressed_handle")
                }
        ) }).unwrap().filter_map(|chat| {chat.ok()}).collect();
        return chats.split_off(offset);
    }

    pub fn get_chat_service_count(&self) -> ChatCounts {
        let mut stmt = self.conn.prepare("SELECT service_name FROM chat").unwrap();
        let chats: Vec<_> = stmt.query_map([], |row| { 
            Ok(row.get_unwrap::<_, String>("service_name"))
        }).unwrap().filter_map(|service| { service.ok() }).collect();
        let mut breakdown = HashMap::new();
        breakdown.insert("iMessage".to_string(), chats.clone().into_iter().filter(|service| *service == "iMessage".to_string()).collect::<Vec<_>>().len());
        breakdown.insert("SMS".to_string(), chats.clone().into_iter().filter(|service| *service == "SMS".to_string()).collect::<Vec<_>>().len());
        ChatCounts { total: chats.len(), breakdown }
    }

    pub fn get_count(&self, table: &str) -> usize {
        let mut stmt = self.conn.prepare(&format!("SELECT COUNT(*) FROM {table}")).unwrap();
        stmt.query_row([], |row| { 
            Ok(row.get_unwrap::<_, usize>("COUNT(*)"))
        }).unwrap()
    }

    pub fn get_chat_participants(&self, chat_row_id: u32) -> Option<Vec<Participant>> {
        let mut stmt = self.conn.prepare("SELECT * FROM chat_handle_join").unwrap();
        return stmt.query_map([], |row| {
            let row_id = row.get("chat_id")?;
            if row_id == chat_row_id {
                return Ok(self.get_participant(row_id));
            }
            return Ok(None);
        }).ok().map(|participants| {
            participants.filter_map(|participant| {
                if let Ok(Some(participant)) = participant {
                    return Some(participant);
                }
                return None;
            }).into_iter().collect()
        });
    }

    pub fn get_participant(&self, row_id: u32) -> Option<Participant> {
        let mut stmt = self.conn.prepare("SELECT * FROM handle WHERE ROWID = ?").unwrap();
        return stmt.query_row([row_id.clone()], |row| {
            Ok(Participant {
                original_rowid: row_id,
                address: row.get("id").unwrap(),
                country: row.get("country").unwrap(),
                uncanonicalized_id: row.get("uncanonicalized_id").ok(),
                service: row.get("service").unwrap(),
            })
        }).ok();
    }

    pub fn get_message_by_guid(&self, message_guid: String, handle: bool, attachments: bool) -> Option<Message> {
        let mut stmt = self.conn.prepare("SELECT * FROM message WHERE guid = ?").unwrap();
        return stmt.query_row([message_guid.clone()], |row| {
            let original_rowid = row.get_unwrap("ROWID");
            let mut stmt = self.conn.prepare("SELECT * FROM chat_message_join WHERE message_id = ?").unwrap();
            let chat_guid = stmt.query_row([original_rowid], |row| {
                let mut stmt = self.conn.prepare("SELECT guid FROM chat WHERE ROWID = ?").unwrap();
                stmt.query_row([row.get_unwrap::<_, u32>("chat_id")], |row| {
                    Ok(row.get_unwrap::<_, String>("guid"))
                })
            }).unwrap();
            let attachments = if attachments {
                self.get_attachments_for_message(original_rowid)
            } else {
                Vec::new()
            };
            Ok(Message::from_row(row, if handle { self.get_participant(row.get_unwrap("handle_id")) } else { None }, attachments, chat_guid, message_guid, original_rowid))
        }).ok();
    }

    pub fn get_attachment_by_guid(&self, attachment_guid: String) -> Option<Attachment> {
        let mut stmt = self.conn.prepare("SELECT * FROM attachment WHERE guid = ?").unwrap();
        return stmt.query_row([attachment_guid.clone()], |row| { 
            let (width, height) = if let Some(file_path) = row.get("filename").ok() {
                let img = ImageReader::open::<String>(file_path).map(|reader| {reader.decode().ok()}).unwrap_or(None);
                if let Some(img) = img {
                    (Some(img.width()), Some(img.height()))
                } else {
                    (None, None) 
                }
            } else {
                    (None, None)
            };
            
            Ok(Attachment {
                original_rowid: row.get_unwrap("ROWID"),
                guid: attachment_guid,
                uti: row.get("uti").ok(),
                mime_type: row.get_unwrap("mime_type"),
                transfer_state: row.get_unwrap("transfer_state"),
                transfer_name: row.get_unwrap("transfer_name"),
                total_bytes: row.get_unwrap("total_bytes"),
                is_outgoing: row.get_unwrap::<&str, i32>("is_outgoing") == 1,
                hide_attachment: row.get_unwrap::<&str, i32>("hide_attachment") == 1,
                is_sticker: row.get_unwrap::<&str, i32>("is_sticker") == 1,
                original_guid: row.get_unwrap("original_guid"),
                metadata: Some("{}".to_string()),
                has_live_photo: false,
                width,
                height,
            })
        }).ok();
    }

    pub fn get_attachments_for_message(&self, message_id: u32) -> Vec<Attachment> {
        let mut stmt = self.conn.prepare("SELECT * FROM message_attachment_join WHERE message_id = ?").unwrap();
        return stmt.query_map([message_id], |row| {
            let mut stmt = self.conn.prepare("SELECT * FROM attachment WHERE ROWID = ?").unwrap();
            stmt.query_row([row.get_unwrap::<&str, u32>("attachment_id")], |row| {
               Ok(self.get_attachment_by_guid(row.get_unwrap("guid")))
            })
        }).unwrap().filter_map(|attachment| {
            if let Ok(Some(attachment)) = attachment {
                return Some(attachment);
            } else {
                return None;
            }
        }).into_iter().collect();
    }

    pub fn get_chat_messages(&self, chat_guid: String, attachments: bool, handle: bool, offset: usize, limit: usize, sort: &str, after: u128, before: u128) -> Option<Vec<Message>> {
        //select m.* from chat c join chat_message_join as cmj on c.rowid=cmj.chat_id join message as m on cmj.message_id=m.rowid where cmj.chat_id=? AND cmj.message_date > ? and cmj.message_date < ? 
        let mut stmt = self.conn.prepare("SELECT ROWID FROM chat WHERE guid = ?").unwrap();
        let mut stmt2 = self.conn.prepare("select m.* from chat c join chat_message_join as cmj on c.rowid=cmj.chat_id join message as m on cmj.message_id=m.rowid where cmj.chat_id=? AND cmj.message_date > ? and cmj.message_date < ?").unwrap();
        let messages = stmt.query_row([chat_guid.clone()], |row| {
            stmt2.query_map((row.get_unwrap::<_, i32>("ROWID"), format!("{after}"), format!("{before}")), |row| {
                let original_rowid = row.get_unwrap("ROWID");
                let attachments = if attachments {
                    self.get_attachments_for_message(original_rowid)
                } else {
                    vec![]
                };
                Ok(Message::from_row(row, if handle { self.get_participant(row.get_unwrap("handle_id")) } else { None }, attachments, chat_guid.clone(), row.get_unwrap("guid"), original_rowid))
            })
        }).ok().map(|messages| {
            let mut messages = messages.filter_map(|message| message.ok()).collect::<Vec<Message>>();
            messages.sort_by(|a, b| b.date_created.cmp(&a.date_created));
            let mut messages = messages.split_off(offset.min(messages.len()));
            let _ = messages.split_off(limit.min(messages.len()));
            messages
        });
        messages
    }

    // pub fn get_chat_messages2(&self, chat_guid: String, attachments: bool, handle: bool, offset: u32, limit: u32, sort: &str, after: u128, before: u128) -> Option<Vec<Message>> {
    //     let mut stmt = self.conn.prepare("SELECT ROWID FROM chat WHERE guid = ?").unwrap();
    //     stmt.query_row([chat_guid], |row| {
    //         println!("found chat");
    //         Ok(self.conn.prepare("SELECT * FROM chat_message_join WHERE chat_id = ?").unwrap().query_map([row.get_unwrap::<_, u32>("ROWID")], |row| {
    //             println!("found messages");
    //             let mut stmt2 = self.conn.prepare("SELECT guid, ROWID FROM message LIMIT ?; AND date > ? AND date < ?; ORDER BY date ?").unwrap();
    //             let mut i = 0;
    //             stmt2.query_row((offset+limit, after.to_string(), before.to_string(), sort), |row| {
    //                 i += 1;
    //                 if i < offset {
    //                     return Ok(None);
    //                 }
    //                 if 
    //                 Ok(Some(self.get_message_by_guid(row.get_unwrap("guid"), handle, attachments)))
    //             })
    //         }).unwrap().filter_map(|message| {
    //             if let Ok(Some(Some(message))) = message {
    //                 return Some(message);
    //             } else {
    //                 return None;
    //             }
    //         }).into_iter().collect())
    //     }).ok()
    // }

    pub fn get_last_chat_message(&self, chat_id: u32) -> Option<Message> {
            self.conn.prepare("SELECT * FROM chat_message_join WHERE chat_id = ? ORDER BY message_date ASC LIMIT 1").unwrap().query_row([chat_id], |row| {
                let mut stmt2 = self.conn.prepare("SELECT guid FROM message WHERE ROWID = ?").unwrap();
                stmt2.query_row([row.get_unwrap::<_, u32>("message_id")], |row| {
                    Ok(self.get_message_by_guid(row.get_unwrap("guid"), true, true))
                })
            }).ok().flatten()
    }
}

mod test {
    #[test]
    fn test_slice() {
        let mut messages = vec![0_i32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let offset = 2;
        let limit = 3;
        let mut messages = messages.split_off(offset.min(messages.len()));
        let _ = messages.split_off(limit.min(messages.len()));
        assert_eq!(messages, vec![2, 3, 4]);
}
}