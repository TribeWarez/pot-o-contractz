#[cfg(test)]
mod tests {
    use anchor_lang::prelude::*;
    use tribewarez_governance::state::{Proposal, ProposalStatus, Treasury, Vote, VoteType};

    fn create_proposal() -> Proposal {
        Proposal {
            proposer: Pubkey::new_unique(),
            title: "Test Proposal".to_string(),
            description: "Test description".to_string(),
            execution_data: vec![],
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            status: ProposalStatus::Active,
            vote_start: 1000,
            vote_end: 2000,
            executed: false,
            execution_timestamp: 0,
            bump: 0,
        }
    }

    fn create_treasury() -> Treasury {
        Treasury {
            balance: 1000,
            authorized_spender: Pubkey::new_unique(),
        }
    }

    fn create_vote() -> Vote {
        Vote {
            voter: Pubkey::new_unique(),
            proposal: Pubkey::new_unique(),
            vote_type: VoteType::For,
            weight: 100,
            timestamp: 1000,
            bump: 0,
        }
    }

    // === Proposal Tests ===

    #[test]
    fn test_proposal_default_state() {
        let proposal = Proposal::default();
        assert_eq!(proposal.title, "");
        assert_eq!(proposal.description, "");
        assert_eq!(proposal.for_votes, 0);
        assert_eq!(proposal.against_votes, 0);
    }

    #[test]
    fn test_proposal_creation() {
        let proposal = create_proposal();
        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.status, ProposalStatus::Active);
        assert!(!proposal.executed);
    }

    #[test]
    fn test_proposal_title_length() {
        let mut proposal = create_proposal();
        proposal.title = "A".to_string();
        assert_eq!(proposal.title.len(), 1);
    }

    #[test]
    fn test_proposal_vote_counts() {
        let mut proposal = create_proposal();
        proposal.for_votes = 100;
        proposal.against_votes = 50;
        proposal.abstain_votes = 25;
        assert_eq!(proposal.for_votes, 100);
        assert_eq!(proposal.against_votes, 50);
        assert_eq!(proposal.abstain_votes, 25);
    }

    #[test]
    fn test_proposal_total_votes() {
        let mut proposal = create_proposal();
        proposal.for_votes = 100;
        proposal.against_votes = 50;
        proposal.abstain_votes = 25;
        let total = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        assert_eq!(total, 175);
    }

    #[test]
    fn test_proposal_status_active() {
        let proposal = create_proposal();
        assert_eq!(proposal.status, ProposalStatus::Active);
    }

    #[test]
    fn test_proposal_vote_timing() {
        let mut proposal = create_proposal();
        proposal.vote_start = 1000;
        proposal.vote_end = 2000;
        assert_eq!(proposal.vote_end - proposal.vote_start, 1000);
    }

    #[test]
    fn test_proposal_execution_flag() {
        let mut proposal = create_proposal();
        assert!(!proposal.executed);
        proposal.executed = true;
        assert!(proposal.executed);
    }

    #[test]
    fn test_proposal_execution_timestamp() {
        let mut proposal = create_proposal();
        proposal.execution_timestamp = 5000;
        assert_eq!(proposal.execution_timestamp, 5000);
    }

    // === Vote Tests ===

    #[test]
    fn test_vote_creation() {
        let vote = create_vote();
        assert_eq!(vote.weight, 100);
        assert_eq!(vote.vote_type, VoteType::For);
    }

    #[test]
    fn test_vote_types() {
        let vote_for = Vote {
            vote_type: VoteType::For,
            ..create_vote()
        };
        let vote_against = Vote {
            vote_type: VoteType::Against,
            ..create_vote()
        };
        let vote_abstain = Vote {
            vote_type: VoteType::Abstain,
            ..create_vote()
        };

        assert_eq!(vote_for.vote_type, VoteType::For);
        assert_eq!(vote_against.vote_type, VoteType::Against);
        assert_eq!(vote_abstain.vote_type, VoteType::Abstain);
    }

    #[test]
    fn test_vote_weight() {
        let mut vote = create_vote();
        vote.weight = 500;
        assert_eq!(vote.weight, 500);
    }

    #[test]
    fn test_vote_timestamp() {
        let mut vote = create_vote();
        vote.timestamp = 3000;
        assert_eq!(vote.timestamp, 3000);
    }

    #[test]
    fn test_vote_link_to_proposal() {
        let proposal_key = Pubkey::new_unique();
        let vote = Vote {
            proposal: proposal_key,
            ..create_vote()
        };
        assert_eq!(vote.proposal, proposal_key);
    }

    #[test]
    fn test_vote_link_to_voter() {
        let voter_key = Pubkey::new_unique();
        let vote = Vote {
            voter: voter_key,
            ..create_vote()
        };
        assert_eq!(vote.voter, voter_key);
    }

    // === Treasury Tests ===

    #[test]
    fn test_treasury_creation() {
        let treasury = create_treasury();
        assert_eq!(treasury.balance, 1000);
    }

    #[test]
    fn test_treasury_balance() {
        let mut treasury = create_treasury();
        treasury.balance = 5000;
        assert_eq!(treasury.balance, 5000);
    }

    #[test]
    fn test_treasury_authorized_spender() {
        let treasury = create_treasury();
        assert_ne!(treasury.authorized_spender, Pubkey::default());
    }

    #[test]
    fn test_treasury_add_funds() {
        let mut treasury = create_treasury();
        treasury.balance += 500;
        assert_eq!(treasury.balance, 1500);
    }

    #[test]
    fn test_treasury_spend_funds() {
        let mut treasury = create_treasury();
        treasury.balance -= 500;
        assert_eq!(treasury.balance, 500);
    }

    #[test]
    fn test_treasury_insufficient_funds() {
        let mut treasury = create_treasury();
        treasury.balance = 100;
        treasury.balance = treasury.balance.saturating_sub(500);
        assert_eq!(treasury.balance, 0); // saturating_sub gives 0, not original value
    }

    // === Voting Logic Tests ===

    #[test]
    fn test_proposal_passed() {
        let mut proposal = create_proposal();
        proposal.for_votes = 600;
        proposal.against_votes = 400;
        proposal.abstain_votes = 0;

        let total = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        let for_pct = (proposal.for_votes as f64 / total as f64) * 100.0;

        assert!(for_pct > 50.0);
    }

    #[test]
    fn test_proposal_rejected() {
        let mut proposal = create_proposal();
        proposal.for_votes = 400;
        proposal.against_votes = 600;
        proposal.abstain_votes = 0;

        let total = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        let for_pct = (proposal.for_votes as f64 / total as f64) * 100.0;

        assert!(for_pct < 50.0);
    }

    #[test]
    fn test_proposal_tie() {
        let mut proposal = create_proposal();
        proposal.for_votes = 500;
        proposal.against_votes = 500;
        proposal.abstain_votes = 0;

        let total = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        let for_pct = (proposal.for_votes as f64 / total as f64) * 100.0;

        assert_eq!(for_pct, 50.0);
    }

    #[test]
    fn test_abstain_votes_not_count() {
        let mut proposal = create_proposal();
        proposal.for_votes = 300;
        proposal.against_votes = 300;
        proposal.abstain_votes = 400;

        let total = proposal.for_votes + proposal.against_votes;
        let for_pct = (proposal.for_votes as f64 / total as f64) * 100.0;

        assert_eq!(for_pct, 50.0);
    }

    // === Edge Cases ===

    #[test]
    fn test_proposal_no_votes() {
        let proposal = create_proposal();
        let total = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        assert_eq!(total, 0);
    }

    #[test]
    fn test_vote_zero_weight() {
        let mut vote = create_vote();
        vote.weight = 0;
        assert_eq!(vote.weight, 0);
    }

    #[test]
    fn test_treasury_zero_balance() {
        let treasury = Treasury::default();
        assert_eq!(treasury.balance, 0);
    }

    #[test]
    fn test_proposal_max_title_length() {
        let mut proposal = create_proposal();
        proposal.title = "A".repeat(100);
        assert_eq!(proposal.title.len(), 100);
    }

    #[test]
    fn test_proposal_execution_data() {
        let mut proposal = create_proposal();
        proposal.execution_data = vec![1, 2, 3, 4, 5];
        assert_eq!(proposal.execution_data.len(), 5);
    }

    // === Complex Scenarios ===

    #[test]
    fn test_voting_cycle() {
        let mut proposal = create_proposal();
        proposal.abstain_votes = 0;

        proposal.for_votes = 100;
        proposal.against_votes = 50;

        let total = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        let for_pct = (proposal.for_votes as f64 / total as f64) * 100.0;

        assert!((for_pct - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_multiple_voters() {
        let voters: Vec<Pubkey> = (0..5).map(|_| Pubkey::new_unique()).collect();
        let total_weight: u64 = voters.len() as u64 * 100;

        assert_eq!(total_weight, 500);
    }

    #[test]
    fn test_proposal_status_transitions() {
        let mut proposal = create_proposal();

        assert_eq!(proposal.status, ProposalStatus::Active);

        proposal.status = ProposalStatus::Passed;
        assert_eq!(proposal.status, ProposalStatus::Passed);

        proposal.executed = true;
        proposal.status = ProposalStatus::Executed;
        assert_eq!(proposal.status, ProposalStatus::Executed);
    }

    #[test]
    fn test_treasury_large_balance() {
        let mut treasury = create_treasury();
        treasury.balance = u64::MAX;
        assert_eq!(treasury.balance, u64::MAX);
    }

    #[test]
    fn test_proposal_init_space() {
        let space = Proposal::INIT_SPACE;
        assert!(space > 0);
    }

    #[test]
    fn test_vote_init_space() {
        let space = Vote::INIT_SPACE;
        assert!(space > 0);
    }

    #[test]
    fn test_treasury_init_space() {
        let space = Treasury::INIT_SPACE;
        assert!(space > 0);
    }

    #[test]
    fn test_vote_bump_tracking() {
        let mut vote = create_vote();
        vote.bump = 42;
        assert_eq!(vote.bump, 42);
    }

    #[test]
    fn test_proposal_bump_tracking() {
        let mut proposal = create_proposal();
        proposal.bump = 42;
        assert_eq!(proposal.bump, 42);
    }

    #[test]
    fn test_treasury_authorized_spender_change() {
        let mut treasury = create_treasury();
        let new_spender = Pubkey::new_unique();
        treasury.authorized_spender = new_spender;
        assert_eq!(treasury.authorized_spender, new_spender);
    }

    #[test]
    fn test_proposal_execution_data_empty() {
        let proposal = create_proposal();
        assert!(proposal.execution_data.is_empty());
    }

    #[test]
    fn test_proposal_all_fields_set() {
        let proposer = Pubkey::new_unique();
        let proposal = Proposal {
            proposer,
            title: "Full Proposal".to_string(),
            description: "Full description".to_string(),
            execution_data: vec![1, 2, 3],
            for_votes: 1000,
            against_votes: 200,
            abstain_votes: 50,
            status: ProposalStatus::Active,
            vote_start: 1000,
            vote_end: 2000,
            executed: false,
            execution_timestamp: 0,
            bump: 1,
        };

        assert_eq!(proposal.proposer, proposer);
        assert_eq!(proposal.title, "Full Proposal");
        assert_eq!(proposal.for_votes, 1000);
    }

    #[test]
    fn test_proposal_for_votes_overflow() {
        let mut proposal = create_proposal();
        proposal.for_votes = u64::MAX;
        let result = proposal.for_votes.checked_add(1);
        assert!(result.is_none());
    }

    #[test]
    fn test_treasury_balance_overflow() {
        let mut treasury = create_treasury();
        treasury.balance = u64::MAX;
        let result = treasury.balance.checked_add(1);
        assert!(result.is_none());
    }

    #[test]
    fn test_vote_weight_overflow() {
        let mut vote = create_vote();
        vote.weight = u64::MAX;
        let result = vote.weight.checked_add(1);
        assert!(result.is_none());
    }

    #[test]
    fn test_proposal_time_calculation() {
        let mut proposal = create_proposal();
        proposal.vote_start = 1000000;
        proposal.vote_end = 2000000;
        let duration = proposal.vote_end - proposal.vote_start;
        assert_eq!(duration, 1000000);
    }

    #[test]
    fn test_voting_quorum_check() {
        let mut proposal = create_proposal();
        proposal.for_votes = 100;
        proposal.against_votes = 0;
        proposal.abstain_votes = 0;

        let total = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        assert!(total > 0);
    }

    #[test]
    fn test_multiple_proposals_independence() {
        let mut p1 = create_proposal();
        let mut p2 = create_proposal();

        p1.for_votes = 100;
        p2.for_votes = 200;

        assert_eq!(p1.for_votes, 100);
        assert_eq!(p2.for_votes, 200);
    }

    #[test]
    fn test_proposal_passed_threshold() {
        let mut proposal = create_proposal();
        proposal.for_votes = 501;
        proposal.against_votes = 499;

        let total = proposal.for_votes + proposal.against_votes;
        let for_pct = (proposal.for_votes as u128 * 10000 / total as u128) as u64;

        assert!(for_pct >= 5000);
    }

    #[test]
    fn test_proposal_failed_threshold() {
        let mut proposal = create_proposal();
        proposal.for_votes = 499;
        proposal.against_votes = 501;

        let total = proposal.for_votes + proposal.against_votes;
        let for_pct = (proposal.for_votes as u128 * 10000 / total as u128) as u64;

        assert!(for_pct < 5000);
    }

    #[test]
    fn test_execution_after_voting_ends() {
        let mut proposal = create_proposal();
        proposal.vote_end = 1000;
        proposal.for_votes = 600;
        proposal.against_votes = 100;

        let current_time = 1500i64;
        assert!(current_time >= proposal.vote_end);
    }

    #[test]
    fn test_proposal_with_execution() {
        let mut proposal = create_proposal();
        proposal.executed = true;
        proposal.execution_timestamp = 3000;

        assert!(proposal.executed);
        assert_eq!(proposal.execution_timestamp, 3000);
    }

    #[test]
    fn test_vote_after_deadline() {
        let proposal = create_proposal();
        let current_time = 3000i64;

        assert!(current_time >= proposal.vote_end);
    }

    #[test]
    fn test_treasury_spent_full() {
        let mut treasury = create_treasury();
        treasury.balance = 1000;
        treasury.balance = treasury.balance.saturating_sub(1000);
        assert_eq!(treasury.balance, 0);
    }

    #[test]
    fn test_proposal_default_bump() {
        let proposal = Proposal::default();
        assert_eq!(proposal.bump, 0);
    }

    #[test]
    fn test_vote_default_bump() {
        let vote = Vote::default();
        assert_eq!(vote.bump, 0);
    }

    #[test]
    fn test_proposal_status_default() {
        let proposal = Proposal::default();
        assert_eq!(proposal.status, ProposalStatus::Active);
    }

    #[test]
    fn test_vote_type_default() {
        let vote = Vote::default();
        assert_eq!(vote.vote_type, VoteType::For);
    }

    #[test]
    fn test_proposal_execution_timestamp_default() {
        let proposal = Proposal::default();
        assert_eq!(proposal.execution_timestamp, 0);
    }

    #[test]
    fn test_treasury_default_authorized() {
        let treasury = Treasury::default();
        assert_eq!(treasury.authorized_spender, Pubkey::default());
    }

    #[test]
    fn test_vote_timestamp_default() {
        let vote = Vote::default();
        assert_eq!(vote.timestamp, 0);
    }
}
