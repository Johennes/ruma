///! Constructors for [predefined push rules].
///!
///! [predefined push rules]: https://matrix.org/docs/spec/client_server/r0.6.1#predefined-rules
use super::{
    Action::*, ConditionalPushRule, PatternedPushRule, PushCondition::*, RoomMemberCountIs,
    Ruleset, Tweak,
};

use ruma_identifiers::UserId;

impl Ruleset {
    /// The list of all [predefined push rules].
    ///
    /// [predefined push rules]: https://matrix.org/docs/spec/client_server/r0.6.1#predefined-rules
    ///
    /// # Parameters
    ///
    /// - `user_id`: the user for which to generate the default rules. Some rules depend on the
    ///   user's ID (for instance those to send notifications when they are mentioned).
    pub fn server_default(user_id: &UserId) -> Self {
        Self {
            content: vec![PatternedPushRule::contains_user_name(user_id)],
            override_: vec![
                ConditionalPushRule::master(),
                ConditionalPushRule::suppress_notices(),
                ConditionalPushRule::invite_for_me(user_id),
                ConditionalPushRule::member_event(),
                ConditionalPushRule::contains_display_name(),
                ConditionalPushRule::tombstone(),
                ConditionalPushRule::roomnotif(),
            ],
            underride: vec![
                ConditionalPushRule::call(),
                ConditionalPushRule::encrypted_room_one_to_one(),
                ConditionalPushRule::room_one_to_one(),
                ConditionalPushRule::message(),
                ConditionalPushRule::encrypted(),
            ],
            ..Default::default()
        }
    }
}

/// Default override push rules
impl ConditionalPushRule {
    /// Matches all events, this can be enabled to turn off all push
    /// notifications other than those generated by override rules set by the user.
    pub fn master() -> Self {
        Self {
            actions: vec![DontNotify],
            default: true,
            enabled: false,
            rule_id: ".m.rule.master".into(),
            conditions: vec![],
        }
    }

    /// Matches messages with a `msgtype` of `notice`.
    pub fn suppress_notices() -> Self {
        Self {
            actions: vec![DontNotify],
            default: true,
            enabled: true,
            rule_id: ".m.rule.suppress_notices".into(),
            conditions: vec![EventMatch {
                key: "content.msgtype".into(),
                pattern: "m.notice".into(),
            }],
        }
    }

    /// Matches any invites to a new room for this user.
    pub fn invite_for_me(user_id: &UserId) -> Self {
        Self {
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".into())),
                SetTweak(Tweak::Highlight(false)),
            ],
            default: true,
            enabled: true,
            rule_id: ".m.rule.invite_for_me".into(),
            conditions: vec![
                EventMatch { key: "type".into(), pattern: "m.room.member".into() },
                EventMatch { key: "content.membership".into(), pattern: "invite".into() },
                EventMatch { key: "state_key".into(), pattern: user_id.to_string() },
            ],
        }
    }

    /// Matches any `m.room.member_event`.
    pub fn member_event() -> Self {
        Self {
            actions: vec![DontNotify],
            default: true,
            enabled: true,
            rule_id: ".m.rule.member_event".into(),
            conditions: vec![EventMatch { key: "type".into(), pattern: "m.room.member".into() }],
        }
    }

    /// Matches any message whose content is unencrypted and contains the user's
    /// current display name in the room in which it was sent.
    pub fn contains_display_name() -> Self {
        Self {
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".into())),
                SetTweak(Tweak::Highlight(true)),
            ],
            default: true,
            enabled: true,
            rule_id: ".m.rule.contains_display_name".into(),
            conditions: vec![ContainsDisplayName],
        }
    }

    /// Matches any state event whose type is `m.room.tombstone`. This
    /// is intended to notify users of a room when it is upgraded,
    /// similar to what an `@room` notification would accomplish.
    pub fn tombstone() -> Self {
        Self {
            actions: vec![Notify, SetTweak(Tweak::Highlight(true))],
            default: true,
            enabled: false,
            rule_id: ".m.rule.tombstone".into(),
            conditions: vec![
                EventMatch { key: "type".into(), pattern: "m.room.tombstone".into() },
                EventMatch { key: "state_key".into(), pattern: "".into() },
            ],
        }
    }

    /// Matches any message whose content is unencrypted and contains the
    /// text `@room`, signifying the whole room should be notified of the event.
    pub fn roomnotif() -> Self {
        Self {
            actions: vec![Notify, SetTweak(Tweak::Highlight(true))],
            default: true,
            enabled: true,
            rule_id: ".m.rule.roomnotif".into(),
            conditions: vec![
                EventMatch { key: "content.body".into(), pattern: "@room".into() },
                SenderNotificationPermission { key: "room".into() },
            ],
        }
    }
}

/// Default content push rules
impl PatternedPushRule {
    /// Matches any message whose content is unencrypted and contains
    /// the local part of the user's Matrix ID, separated by word boundaries.
    pub fn contains_user_name(user_id: &UserId) -> Self {
        Self {
            rule_id: ".m.rules.contains_user_name".into(),
            enabled: true,
            default: true,
            pattern: user_id.localpart().into(),
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".into())),
                SetTweak(Tweak::Highlight(true)),
            ],
        }
    }
}

/// Default underrides push rules
impl ConditionalPushRule {
    /// Matches any incoming VOIP call.
    pub fn call() -> Self {
        Self {
            rule_id: ".m.rules.call".into(),
            default: true,
            enabled: true,
            conditions: vec![EventMatch { key: "type".into(), pattern: "m.call.invite".into() }],
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("ring".into())),
                SetTweak(Tweak::Highlight(false)),
            ],
        }
    }

    /// Matches any encrypted event sent in a room with exactly
    /// two members. Unlike other push rules, this rule cannot be
    /// matched against the content of the event by nature of it
    /// being encrypted. This causes the rule to be an "all or
    /// nothing" match where it either matches all events that are
    /// encrypted (in 1:1 rooms) or none.
    pub fn encrypted_room_one_to_one() -> Self {
        Self {
            rule_id: ".m.rules.encrypted_room_one_to_one".into(),
            default: true,
            enabled: true,
            conditions: vec![
                RoomMemberCount { is: RoomMemberCountIs::from(js_int::uint!(2)) },
                EventMatch { key: "type".into(), pattern: "m.room.encrypted".into() },
            ],
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".into())),
                SetTweak(Tweak::Highlight(false)),
            ],
        }
    }

    /// Matches any message sent in a room with exactly two members.
    pub fn room_one_to_one() -> Self {
        Self {
            rule_id: ".m.rules.room_one_to_one".into(),
            default: true,
            enabled: true,
            conditions: vec![
                RoomMemberCount { is: RoomMemberCountIs::from(js_int::uint!(2)) },
                EventMatch { key: "type".into(), pattern: "m.room.message".into() },
            ],
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".into())),
                SetTweak(Tweak::Highlight(false)),
            ],
        }
    }

    /// Matches all chat messages.
    pub fn message() -> Self {
        Self {
            rule_id: ".m.rules.message".into(),
            default: true,
            enabled: true,
            conditions: vec![EventMatch { key: "type".into(), pattern: "m.room.message".into() }],
            actions: vec![Notify, SetTweak(Tweak::Highlight(false))],
        }
    }

    /// Matches all encrypted events. Unlike other push rules,
    /// this rule cannot be matched against the content of the
    /// event by nature of it being encrypted. This causes the
    /// rule to be an "all or nothing" match where it either matches
    /// all events that are encrypted (in group rooms) or none.
    pub fn encrypted() -> Self {
        Self {
            rule_id: ".m.rules.encrypted".into(),
            default: true,
            enabled: true,
            conditions: vec![EventMatch { key: "type".into(), pattern: "m.room.encrypted".into() }],
            actions: vec![Notify, SetTweak(Tweak::Highlight(false))],
        }
    }
}
