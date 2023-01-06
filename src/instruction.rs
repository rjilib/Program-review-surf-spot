use solana_program::program_error::ProgramError;
use borsh::BorshDeserialize;

pub enum InstructionSpotSurf{
    InitSpotSurfInfo{
        name: String,
        rate: u16,
        review: String,
    },
    UpdateSpotSurfInfo{
        name: String,
        rate: u16,
        review: String,
    },
    AddComment{
        comment: String
    },
    InitMint
}

#[derive(BorshDeserialize, Debug)]
struct SpotSurfInfoPayload {
    pub name: String,
    pub rate: u16,
    pub review: String,
}

#[derive(BorshDeserialize, Debug)]
struct CommentPayload {
    pub comment: String,
}

impl InstructionSpotSurf {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) =  input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match variant {
            0 => {
                let payload = SpotSurfInfoPayload::try_from_slice(rest).unwrap();
                Self::InitSpotSurfInfo { 
                    name: payload.name, 
                    rate: payload.rate, 
                    review: payload.review 
                }
            }
            1 => {
                let payload = SpotSurfInfoPayload::try_from_slice(rest).unwrap();
                Self::UpdateSpotSurfInfo { 
                    name: payload.name, 
                    rate: payload.rate, 
                    review: payload.review 
                }
            }
            2 => {
                let payload = CommentPayload::try_from_slice(rest).unwrap();
                Self::AddComment { comment: payload.comment }
            }
            3 => Self::InitMint,
            _ => return Err(ProgramError::InvalidInstructionData)
            
        })
    }
}