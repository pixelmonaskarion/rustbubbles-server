use serde::Serialize;

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
}

#[derive(Debug, Serialize)]
pub struct Participant {
    pub original_rowid: u32,
    pub address: String,
    pub country: String,
    pub uncanonicalized_id: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Message {
    #[serde(rename = "originalROWID")]
    pub original_rowid: u32,
    pub guid: String,
    pub text: Option<String>,
    pub handle: Option<Participant>,
    pub chat_guid: String,
    pub attachments: Vec<Attachment>,
    #[serde(rename = "groupActionType")]
    pub group_action_type: u32,
    pub item_type: u32,
    #[serde(rename = "otherHandle")]
    pub other_handle: Option<u32>,
    #[serde(rename = "dateCreated")]
    pub date_created: u128,
}

#[derive(Debug, Serialize)]
pub struct Attachment {
    pub original_rowid: u32,
    pub guid: String,
    pub uti: String,
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