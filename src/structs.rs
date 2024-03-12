use rusqlite::Row;
use serde::Serialize;

use crate::util::apple_to_unix;

#[derive(Debug, Serialize)]
pub struct Chat {
    #[serde(rename = "originalROWID")]
    pub original_rowid: u32,
    pub guid: String,
    pub participants: Option<Vec<Participant>>,
    #[serde(rename = "lastMessage")]
    pub last_message: Option<Message>,
    pub style: u32,
    #[serde(rename = "chatIdentifier")]
    pub chat_identifier: String,
    #[serde(rename = "isArchived")]
    pub is_archived: bool,
    #[serde(rename = "isFiltered")]
    pub is_filtered: bool,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "lastAddressedHandle")]
    pub last_addressed_handle: String,
}

#[derive(Debug, Serialize)]
pub struct Participant {
    #[serde(rename = "originalROWID")]
    pub original_rowid: u32,
    pub address: String,
    pub country: String,
    #[serde(rename = "uncanonicalizedId")]
    pub uncanonicalized_id: Option<u32>,
    pub service: String,
}

#[derive(Debug, Serialize)]
pub struct Message {
    #[serde(rename = "originalROWID")]
    pub original_rowid: u32,
    pub guid: String,
    pub text: Option<String>,
    pub handle: Option<Participant>,
    #[serde(rename = "handleId")]
    pub handle_id: u32,
    pub subject: Option<String>,
    pub error: i32,
    #[serde(skip_serializing)]
    pub chat_guid: String,
    pub attachments: Vec<Attachment>,
    #[serde(rename = "groupActionType")]
    pub group_action_type: u32,
    #[serde(rename = "itemType")]
    pub item_type: u32,
    #[serde(rename = "otherHandle")]
    pub other_handle: Option<u32>,
    #[serde(rename = "dateCreated")]
    pub date_created: u128,
    #[serde(rename = "dateRead")]
    pub date_read: u128,
    #[serde(rename = "dateDelivered")]
    pub date_delivered: u128,
    #[serde(rename = "isFromMe")]
    pub is_from_me: bool,
    #[serde(rename = "hasDdResults")]
    pub has_dd_results: bool,
    #[serde(rename = "groupTitle")]
    pub group_title: Option<String>,
    #[serde(rename = "associatedMessageGuid")]
    pub associated_message_guid: Option<String>,
    #[serde(rename = "associatedMessageType")]
    pub associated_message_type: Option<String>,
    #[serde(rename = "expressiveSendStyleId")]
    pub expressive_send_style_id: Option<String>,
    #[serde(rename = "threadOriginatorGuid")]
    pub thread_originator_guid: Option<String>,
    #[serde(rename = "threadOriginatorPart")]
    pub thread_originator_part: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "isDelayed")]
    pub is_delayed: bool,
    #[serde(rename = "isAutoReply")]
    pub is_auto_reply: bool,
    #[serde(rename = "isSystemMessage")]
    pub is_system_message: bool,
    #[serde(rename = "isServiceMessage")]
    pub is_service_message: bool,
    #[serde(rename = "isForward")]
    pub is_forward: bool,
    #[serde(rename = "isCorrupt")]
    pub is_corrupt: bool,
    #[serde(rename = "datePlayed")]
    pub date_played: u128,
    #[serde(rename = "cacheRoomnames")]
    pub cache_roomnames: Option<String>,
    #[serde(rename = "isSpam")]
    pub is_spam: bool,
    #[serde(rename = "isAudioMessage")]
    pub is_audio_message: bool,
    #[serde(rename = "replyToGuid")]
    pub reply_to_guid: Option<String>,
    #[serde(rename = "shareStatus")]
    pub share_status: i32,
    #[serde(rename = "shareDirection")]
    pub share_direction: i32,
    #[serde(rename = "wasDeliveredQuietly")]
    pub was_delivered_quietly: bool,
    #[serde(rename = "didNotifyRecipient")]
    pub did_notify_recipient: bool,
    // pub chats: Vec<Chat>,
}

impl Message {
    pub fn from_row(row: &Row, handle: Option<Participant>, attachments: Vec<Attachment>, chat_guid: String, guid: String, original_rowid: u32) -> Self {
        Self {
            original_rowid,
            guid,
            text: row.get_unwrap("text"),
            date_created: apple_to_unix(row.get_unwrap::<_, usize>("date") as u128)/1000000,
            handle,
            chat_guid,
            attachments,
            group_action_type: row.get_unwrap("group_action_type"),
            item_type: row.get_unwrap("item_type"),
            other_handle: row.get("other_handle").ok(),
            is_from_me: row.get_unwrap("is_from_me"),
            associated_message_guid: row.get("associated_message_guid").ok(),
            associated_message_type: row.get("associated_message_type").ok(),
            cache_roomnames: row.get_unwrap("cache_roomnames"),
            country: row.get("country").ok(),
            date_delivered: apple_to_unix(row.get_unwrap::<_, usize>("date_delivered") as u128)/1000000,
            date_played: apple_to_unix(row.get_unwrap::<_, usize>("date_played") as u128)/1000000,
            date_read: apple_to_unix(row.get_unwrap::<_, usize>("date_read") as u128)/1000000,
            did_notify_recipient: row.get_unwrap("did_notify_recipient"),
            error: row.get_unwrap("error"),
            expressive_send_style_id: row.get("expressive_send_style_id").ok(),
            group_title: row.get("group_title").ok(),
            handle_id: row.get_unwrap("handle_id"),
            subject: row.get("subject").ok(),
            has_dd_results: row.get_unwrap("has_dd_results"),
            is_audio_message: row.get_unwrap("is_audio_message"),
            is_auto_reply: row.get_unwrap("is_auto_reply"),
            is_corrupt: row.get_unwrap("is_corrupt"),
            is_delayed: row.get_unwrap("is_delayed"),
            is_forward: row.get_unwrap("is_forward"),
            is_service_message: row.get_unwrap("is_service_message"),
            is_spam: row.get_unwrap("is_spam"),
            is_system_message: row.get_unwrap("is_system_message"),
            reply_to_guid: row.get("reply_to_guid").ok(),
            share_direction: row.get_unwrap("share_direction"),
            share_status: row.get_unwrap("share_status"),
            thread_originator_guid: row.get("thread_originator_guid").ok(),
            thread_originator_part: row.get("thread_originator_part").ok(),
            was_delivered_quietly: row.get_unwrap("was_delivered_quietly"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Attachment {
    pub original_rowid: u32,
    pub guid: String,
    pub uti: Option<String>,
    pub mime_type: Option<String>,
    pub transfer_name: String,
    pub total_bytes: u32,
    pub transfer_state: u32,
    pub is_outgoing: bool,
    pub hide_attachment: bool,
    pub is_sticker: bool,
    pub original_guid: String,
    pub has_live_photo: bool,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub metadata: Option<String>,
}