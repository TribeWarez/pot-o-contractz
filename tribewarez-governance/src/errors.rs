use anchor_lang::prelude::*;

#[error_code]
pub enum GovernanceError {
    #[msg("Invalid proposal title")]
    InvalidTitle,

    #[msg("Title too long")]
    TitleTooLong,

    #[msg("Description too long")]
    DescriptionTooLong,

    #[msg("Proposal not active")]
    ProposalNotActive,

    #[msg("Voting has ended")]
    VotingEnded,

    #[msg("Invalid vote weight")]
    InvalidVoteWeight,

    #[msg("Voting has not ended")]
    VotingNotEnded,

    #[msg("No votes cast")]
    NoVotes,

    #[msg("Proposal rejected")]
    ProposalRejected,

    #[msg("Already executed")]
    AlreadyExecuted,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Unauthorized")]
    Unauthorized,
}
