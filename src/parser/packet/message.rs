#![allow(clippy::upper_case_acronyms)] // Needed for `NetMessage::NOP`

use paste::paste;
use protobuf::{CodedInputStream, Enum};
use tracing::{instrument, trace};

use crate::error::Result;
use crate::messages::*;

macro_rules! create_message_type_enum {
    ($msg_type:ident, $($msg:ident),*) => {
        paste! {
            #[derive(Debug)]
            pub enum [<$msg_type:camel Message>] {
                $(
                    $msg([<C $msg_type:upper Msg_ $msg>]),
                )*
            }
        }
    }
}

macro_rules! create_message_enum {
    ($($msg_type:ident = [$($msg:ident),*]),*) => {
        paste! {
            #[derive(Debug)]
            pub enum Message {
                Invalid(i32),
                $(
                    [<$msg_type:camel>]([<$msg_type:camel Message>]),
                )*
            }
        }

        $(
            create_message_type_enum!($msg_type, $($msg),*);
        )*
    }
}

macro_rules! create_message_impl {
    (
        // Needed for hygienic reasons:
        // `reader` needs to be in the same context as of this macro for it to
        // be used in our custom blocks
        $reader:ident,
        // Sequence of "message type" associated with a list of the messages
        // for this type
        $(
            $msg_type:ident = [
                $(
                    $msg:ident
                ),*
            ]
        ),*;
        usr = [
            $(
                $usr_msg:ident
            ),*
        ]
    ) => {
        create_message_enum!(
            $(
                $msg_type = [$($msg),*]
            ),*,
            usr = [$($usr_msg),*]
        );

        impl Message {
            #[instrument(level = "trace", skip($reader))]
            pub(crate) fn try_new($reader: &mut CodedInputStream) -> Result<Message> {
                let command = $reader.read_int32()?;

                // Handle "User message" before anything else
                //
                // User Messages are particular, because they are SVC messages
                // that contains a User message. This piece of code does the
                // "conversion".
                if command == SVC_Messages::svc_UserMessage.value() {
                    let svc_message: CSVCMsg_UserMessage = $reader.read_message()?;

                    // The data from protobuf's `CSVCMsg_UserMessage` contains
                    // a `CUSRMsg_x`, so we need a new reader on those data to
                    // read our new message.
                    let mut data = svc_message.msg_data();
                    let mut temp_reader = CodedInputStream::new(&mut data);

                    paste! {
                        if let Some(message_type) = USR_Messages::from_i32(svc_message.msg_type()) {
                            return Ok(match message_type {
                                $(
                                    USR_Messages::[<usr_ $usr_msg>] =>
                                        Message::Usr(UsrMessage::$usr_msg(temp_reader.read_message()?)),
                                )*
                                    // Fallback for unhandled User messages
                                    _ => Message::Svc(SvcMessage::UserMessage(svc_message.clone())),
                                });
                            }
                    }
                }

                $(
                    paste! {
                        if let Some(message_type) =
                            [<$msg_type:upper _Messages>]::from_i32(command) {
                                return Ok(Message::[<$msg_type:camel>](
                                    match message_type {
                                        $(
                                            [<$msg_type:upper _Messages>]::[<$msg_type _ $msg>] =>
                                                [<$msg_type:camel Message>]::$msg($reader.read_message()?)
                                        ),*
                                }));
                        }
                    }
                )*

                // Else
                trace!("invalid message found: {}, skipping it", command);

                let size = $reader.read_fixed32()?;
                $reader.skip_raw_bytes(size)?;

                Ok(Message::Invalid(command))
            }
        }
    }
}

create_message_impl!(
    reader,
    net = [
        NOP,
        Disconnect,
        File,
        SplitScreenUser,
        Tick,
        StringCmd,
        SetConVar,
        SignonState,
        PlayerAvatarData
    ],
    svc = [
        ServerInfo,
        SendTable,
        ClassInfo,
        SetPause,
        CreateStringTable,
        UpdateStringTable,
        VoiceInit,
        VoiceData,
        Print,
        Sounds,
        SetView,
        FixAngle,
        CrosshairAngle,
        BSPDecal,
        SplitScreen,
        UserMessage,
        EntityMessage,
        GameEvent,
        PacketEntities,
        TempEntities,
        Prefetch,
        Menu,
        GameEventList,
        GetCvarValue,
        PaintmapData,
        CmdKeyValues,
        EncryptedData,
        HltvReplay,
        BroadcastCommand
    ];
    usr = [
        VGUIMenu,
        Geiger,
        Train,
        HudText,
        SayText,
        SayText2,
        TextMsg,
        HudMsg,
        ResetHud,
        GameTitle,
        Shake,
        Fade,
        Rumble,
        CloseCaption,
        CloseCaptionDirect,
        SendAudio,
        RawAudio,
        VoiceMask,
        RequestState,
        Damage,
        RadioText,
        HintText,
        KeyHintText,
        ProcessSpottedEntityUpdate,
        ReloadEffect,
        AdjustMoney,
        StopSpectatorMode,
        KillCam,
        DesiredTimescale,
        CurrentTimescale,
        AchievementEvent,
        MatchEndConditions,
        DisconnectToLobby,
        PlayerStatsUpdate,
        DisplayInventory,
        WarmupHasEnded,
        ClientInfo,
        XRankGet,
        XRankUpd,
        CallVoteFailed,
        VoteStart,
        VotePass,
        VoteFailed,
        VoteSetup,
        ServerRankRevealAll,
        SendLastKillerDamageToClient,
        ServerRankUpdate,
        ItemPickup,
        ShowMenu,
        BarTime,
        AmmoDenied,
        MarkAchievement,
        MatchStatsUpdate,
        ItemDrop,
        GlowPropTurnOff,
        SendPlayerItemDrops,
        RoundBackupFilenames,
        SendPlayerItemFound,
        ReportHit,
        XpUpdate,
        QuestProgress,
        ScoreLeaderboardData,
        PlayerDecalDigitalSignature,
        WeaponSound,
        UpdateScreenHealthBar,
        EntityOutlineHighlight,
        SSUI,
        SurvivalStats,
        EndOfMatchAllPlayersData,
        RoundImpactScoreData,
        CurrentRoundOdds,
        DeepStats,
        UtilMessage,
        UtilMessageResponse
    ]
);
