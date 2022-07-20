use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum IntroInstruction {
    InitUserInput { name: String, message: String },
    UpdateStudentIntro { name: String, message: String },
    AddReply { reply: String },
}

#[derive(BorshDeserialize, Debug)]
struct StudentIntroPayload {
    name: String,
    message: String,
}

#[derive(BorshDeserialize)]
struct ReplyPayload {
    reply: String,
}

impl IntroInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match variant {
            0 => {
                let payload = StudentIntroPayload::try_from_slice(rest).unwrap();
                Self::InitUserInput {
                    name: payload.name,
                    message: payload.message,
                }
            }
            1 => {
                let payload = StudentIntroPayload::try_from_slice(rest).unwrap();
                Self::UpdateStudentIntro {
                    name: payload.name,
                    message: payload.message,
                }
            }
            2 => {
                let payload = ReplyPayload::try_from_slice(rest).unwrap();
                Self::AddReply {
                    reply: payload.reply,
                }
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
