use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum IntroInstruction {
    InitUserInput { name: String, message: String },
    UpdateStudentIntro { name: String, message: String },
}

#[derive(BorshDeserialize, Debug)]
struct StudentIntroPayload {
    name: String,
    message: String,
}

impl IntroInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&discriminator, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        let payload = StudentIntroPayload::try_from_slice(rest)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match discriminator {
            0 => Ok(Self::InitUserInput {
                name: payload.name,
                message: payload.message,
            }),
            1 => Ok(Self::UpdateStudentIntro {
                name: payload.name,
                message: payload.message,
            }),
            _ => return Err(ProgramError::InvalidInstructionData),
        }
    }
}
