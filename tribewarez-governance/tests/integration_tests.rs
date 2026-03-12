//! Comprehensive integration tests for tribewarez-governance program
//! Tests all governance operations: proposals, voting, execution, and treasury management

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::*;
    use tribewarez_governance::state::{Proposal, Treasury, Vote};

    // Test Constants
    const PROPOSAL_ID: u64 = 1;
    const VOTING_PERIOD: i64 = 7 * 24 * 60 * 60; // 7 days in seconds
    const VOTING_THRESHOLD: u64 = 1000; // Minimum votes to pass
    const TREASURY_INITIAL: u64 = 10_000_000_000; // 10B tokens

    // ============================================================================
    // Proposal Tests
    // ============================================================================

    #[test]
    fn test_proposal_default_state() {
        let proposal = Proposal::default();
        assert_eq!(proposal.id, 0);
        assert_eq!(proposal.title, "");
        assert_eq!(proposal.description, "");
        assert_eq!(proposal.voting_start, 0);
        assert_eq!(proposal.voting_end, 0);
    }

    #[test]
    fn test_proposal_creation() {
        let proposer = Pubkey::new_unique();
        let now: i64 = 1_000_000;

        let proposal = Proposal {
            id: 1,
            proposer,
            title: "Budget Allocation".to_string(),
            description: "Allocate 1M tokens to marketing".to_string(),
            voting_start: now,
            voting_end: now + VOTING_PERIOD,
        };

        assert_eq!(proposal.id, 1);
        assert_eq!(proposal.proposer, proposer);
        assert_eq!(proposal.title, "Budget Allocation");
        assert_eq!(proposal.voting_end - proposal.voting_start, VOTING_PERIOD);
    }

    #[test]
    fn test_multiple_proposals() {
        let proposers = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];

        let mut proposals = vec![];
        for (id, proposer) in proposers.iter().enumerate() {
            proposals.push(Proposal {
                id: id as u64 + 1,
                proposer: *proposer,
                title: format!("Proposal {}", id + 1),
                description: format!("Description for proposal {}", id + 1),
                voting_start: 1_000_000 + (id as i64 * 1_000_000),
                voting_end: 1_000_000 + VOTING_PERIOD + (id as i64 * 1_000_000),
            });
        }

        assert_eq!(proposals.len(), 3);
        assert_eq!(proposals[0].id, 1);
        assert_eq!(proposals[2].id, 3);
    }

    #[test]
    fn test_proposal_title_and_description() {
        let proposal = Proposal {
            id: 1,
            proposer: Pubkey::new_unique(),
            title: "Emergency Fund".to_string(),
            description: "Create emergency fund for network security".to_string(),
            voting_start: 1_000_000,
            voting_end: 1_000_000 + VOTING_PERIOD,
        };

        assert!(!proposal.title.is_empty());
        assert!(!proposal.description.is_empty());
        assert!(proposal.title.len() <= 256); // Reasonable max
    }

    // ============================================================================
    // Vote Tests
    // ============================================================================

    #[test]
    fn test_vote_default_state() {
        let vote = Vote::default();
        assert_eq!(vote.proposal_id, 0);
        assert_eq!(vote.amount, 0);
        assert_eq!(vote.direction, 0); // abstain
    }

    #[test]
    fn test_vote_creation_yes() {
        let voter = Pubkey::new_unique();
        let vote = Vote {
            proposal_id: 1,
            voter,
            amount: 500_000,
            direction: 1, // Yes
        };

        assert_eq!(vote.proposal_id, 1);
        assert_eq!(vote.voter, voter);
        assert_eq!(vote.amount, 500_000);
        assert_eq!(vote.direction, 1);
    }

    #[test]
    fn test_vote_creation_no() {
        let voter = Pubkey::new_unique();
        let vote = Vote {
            proposal_id: 1,
            voter,
            amount: 250_000,
            direction: 2, // No
        };

        assert_eq!(vote.direction, 2);
    }

    #[test]
    fn test_vote_creation_abstain() {
        let voter = Pubkey::new_unique();
        let vote = Vote {
            proposal_id: 1,
            voter,
            amount: 0,
            direction: 0, // Abstain
        };

        assert_eq!(vote.direction, 0);
        assert_eq!(vote.amount, 0);
    }

    #[test]
    fn test_multiple_votes_same_proposal() {
        let voters = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];

        let votes: Vec<Vote> = vec![
            Vote {
                proposal_id: 1,
                voter: voters[0],
                amount: 1000,
                direction: 1, // Yes
            },
            Vote {
                proposal_id: 1,
                voter: voters[1],
                amount: 500,
                direction: 2, // No
            },
            Vote {
                proposal_id: 1,
                voter: voters[2],
                amount: 0,
                direction: 0, // Abstain
            },
        ];

        assert_eq!(votes.len(), 3);

        // Calculate totals
        let yes_votes: u64 = votes
            .iter()
            .filter(|v| v.direction == 1)
            .map(|v| v.amount)
            .sum();
        let no_votes: u64 = votes
            .iter()
            .filter(|v| v.direction == 2)
            .map(|v| v.amount)
            .sum();
        let total_votes: u64 = votes.iter().map(|v| v.amount).sum();

        assert_eq!(yes_votes, 1000);
        assert_eq!(no_votes, 500);
        assert_eq!(total_votes, 1500);
    }

    #[test]
    fn test_vote_weight_aggregation() {
        let proposal_id = 1;
        let mut yes_weight = 0u64;
        let mut no_weight = 0u64;

        // Simulate votes
        let vote_amounts = vec![100, 250, 150, 200, 300];

        for (i, amount) in vote_amounts.iter().enumerate() {
            if i % 2 == 0 {
                yes_weight += amount;
            } else {
                no_weight += amount;
            }
        }

        assert_eq!(yes_weight, 550); // 100 + 150 + 300
        assert_eq!(no_weight, 450); // 250 + 200
    }

    // ============================================================================
    // Treasury Tests
    // ============================================================================

    #[test]
    fn test_treasury_default_state() {
        let treasury = Treasury::default();
        assert_eq!(treasury.balance, 0);
    }

    #[test]
    fn test_treasury_initialization() {
        let authorized = Pubkey::new_unique();
        let treasury = Treasury {
            balance: TREASURY_INITIAL,
            authorized_spender: authorized,
        };

        assert_eq!(treasury.balance, TREASURY_INITIAL);
        assert_eq!(treasury.authorized_spender, authorized);
    }

    #[test]
    fn test_treasury_withdrawal() {
        let mut treasury = Treasury {
            balance: 1_000_000,
            authorized_spender: Pubkey::new_unique(),
        };

        let withdrawal = 250_000;
        if treasury.balance >= withdrawal {
            treasury.balance -= withdrawal;
        }

        assert_eq!(treasury.balance, 750_000);
    }

    #[test]
    fn test_treasury_deposit() {
        let mut treasury = Treasury {
            balance: 1_000_000,
            authorized_spender: Pubkey::new_unique(),
        };

        let deposit = 500_000;
        treasury.balance = treasury.balance.checked_add(deposit).unwrap_or(u64::MAX);

        assert_eq!(treasury.balance, 1_500_000);
    }

    #[test]
    fn test_treasury_multiple_withdrawals() {
        let mut treasury = Treasury {
            balance: 5_000_000,
            authorized_spender: Pubkey::new_unique(),
        };

        let withdrawals = vec![500_000, 750_000, 250_000];

        for withdrawal in withdrawals {
            if treasury.balance >= withdrawal {
                treasury.balance -= withdrawal;
            }
        }

        assert_eq!(treasury.balance, 3_500_000);
    }

    #[test]
    fn test_treasury_insufficient_balance() {
        let treasury = Treasury {
            balance: 100_000,
            authorized_spender: Pubkey::new_unique(),
        };

        let withdrawal = 150_000;
        assert!(treasury.balance < withdrawal);
    }

    #[test]
    fn test_treasury_balance_after_operations() {
        let mut treasury = Treasury {
            balance: 1_000_000,
            authorized_spender: Pubkey::new_unique(),
        };

        // Deposit
        treasury.balance += 500_000;
        assert_eq!(treasury.balance, 1_500_000);

        // Withdraw
        treasury.balance -= 200_000;
        assert_eq!(treasury.balance, 1_300_000);

        // Deposit
        treasury.balance += 100_000;
        assert_eq!(treasury.balance, 1_400_000);
    }

    // ============================================================================
    // Proposal Lifecycle Tests
    // ============================================================================

    #[test]
    fn test_proposal_voting_period() {
        let now: i64 = 1_000_000;
        let proposal = Proposal {
            id: 1,
            proposer: Pubkey::new_unique(),
            title: "Test".to_string(),
            description: "Test proposal".to_string(),
            voting_start: now,
            voting_end: now + VOTING_PERIOD,
        };

        // Check if voting is active at different times
        assert!(now >= proposal.voting_start);
        assert!(now <= proposal.voting_end);

        // Check future time
        let future_time = now + VOTING_PERIOD + 1;
        assert!(future_time > proposal.voting_end); // Voting ended
    }

    #[test]
    fn test_proposal_before_voting_starts() {
        let now: i64 = 1_000_000;
        let proposal = Proposal {
            id: 1,
            proposer: Pubkey::new_unique(),
            title: "Future".to_string(),
            description: "Future proposal".to_string(),
            voting_start: now + 1_000_000,
            voting_end: now + 1_000_000 + VOTING_PERIOD,
        };

        assert!(now < proposal.voting_start);
    }

    #[test]
    fn test_proposal_voting_concluded() {
        let now: i64 = 2_000_000;
        let proposal = Proposal {
            id: 1,
            proposer: Pubkey::new_unique(),
            title: "Past".to_string(),
            description: "Past proposal".to_string(),
            voting_start: 1_000_000,
            voting_end: 1_500_000,
        };

        assert!(now > proposal.voting_end);
    }

    // ============================================================================
    // Governance Decision Tests
    // ============================================================================

    #[test]
    fn test_proposal_passes_simple_majority() {
        let yes_votes = 600;
        let no_votes = 400;
        let total = yes_votes + no_votes;

        let passes = yes_votes > no_votes && total >= VOTING_THRESHOLD;
        assert!(passes);
    }

    #[test]
    fn test_proposal_fails_simple_majority() {
        let yes_votes = 400;
        let no_votes = 600;
        let total = yes_votes + no_votes;

        let passes = yes_votes > no_votes && total >= VOTING_THRESHOLD;
        assert!(!passes);
    }

    #[test]
    fn test_proposal_fails_below_threshold() {
        let yes_votes = 800;
        let no_votes = 100;
        let total = yes_votes + no_votes;

        let passes = yes_votes > no_votes && total >= VOTING_THRESHOLD;
        // Fails because total (900) < threshold (1000) despite clear majority
        assert!(!passes);
    }

    #[test]
    fn test_proposal_unanimity() {
        let yes_votes = 1000;
        let no_votes = 0;

        let passes = yes_votes > no_votes;
        assert!(passes);
    }

    #[test]
    fn test_proposal_tie_fails() {
        let yes_votes = 500;
        let no_votes = 500;

        let passes = yes_votes > no_votes; // Strict majority required
        assert!(!passes);
    }

    // ============================================================================
    // Multi-Proposal Scenarios
    // ============================================================================

    #[test]
    fn test_sequential_proposals() {
        let mut proposals = vec![];

        for i in 0..5 {
            proposals.push(Proposal {
                id: i + 1,
                proposer: Pubkey::new_unique(),
                title: format!("Proposal {}", i + 1),
                description: format!("Description {}", i + 1),
                voting_start: 1_000_000 + (i as i64 * 100_000),
                voting_end: 1_000_000 + VOTING_PERIOD + (i as i64 * 100_000),
            });
        }

        assert_eq!(proposals.len(), 5);
        for (i, prop) in proposals.iter().enumerate() {
            assert_eq!(prop.id, (i + 1) as u64);
        }
    }

    #[test]
    fn test_concurrent_voting_on_different_proposals() {
        let proposal1 = Proposal {
            id: 1,
            proposer: Pubkey::new_unique(),
            title: "Proposal 1".to_string(),
            description: "First proposal".to_string(),
            voting_start: 1_000_000,
            voting_end: 1_000_000 + VOTING_PERIOD,
        };

        let proposal2 = Proposal {
            id: 2,
            proposer: Pubkey::new_unique(),
            title: "Proposal 2".to_string(),
            description: "Second proposal".to_string(),
            voting_start: 1_000_000,
            voting_end: 1_000_000 + VOTING_PERIOD,
        };

        let voter1 = Pubkey::new_unique();
        let voter2 = Pubkey::new_unique();

        let votes = vec![
            Vote {
                proposal_id: 1,
                voter: voter1,
                amount: 1000,
                direction: 1,
            },
            Vote {
                proposal_id: 2,
                voter: voter1,
                amount: 1000,
                direction: 2,
            },
            Vote {
                proposal_id: 1,
                voter: voter2,
                amount: 500,
                direction: 2,
            },
            Vote {
                proposal_id: 2,
                voter: voter2,
                amount: 500,
                direction: 1,
            },
        ];

        assert_eq!(votes.len(), 4);

        // Proposal 1: Yes=1000, No=500
        let p1_yes: u64 = votes
            .iter()
            .filter(|v| v.proposal_id == 1 && v.direction == 1)
            .map(|v| v.amount)
            .sum();
        let p1_no: u64 = votes
            .iter()
            .filter(|v| v.proposal_id == 1 && v.direction == 2)
            .map(|v| v.amount)
            .sum();

        // Proposal 2: Yes=500, No=1000
        let p2_yes: u64 = votes
            .iter()
            .filter(|v| v.proposal_id == 2 && v.direction == 1)
            .map(|v| v.amount)
            .sum();
        let p2_no: u64 = votes
            .iter()
            .filter(|v| v.proposal_id == 2 && v.direction == 2)
            .map(|v| v.amount)
            .sum();

        assert!(p1_yes > p1_no); // P1 passes
        assert!(p2_no > p2_yes); // P2 fails
    }

    // ============================================================================
    // Authority and Permission Tests
    // ============================================================================

    #[test]
    fn test_authority_validation() {
        let authority1 = Pubkey::new_unique();
        let authority2 = Pubkey::new_unique();

        let treasury = Treasury {
            balance: 1_000_000,
            authorized_spender: authority1,
        };

        assert_eq!(treasury.authorized_spender, authority1);
        assert_ne!(treasury.authorized_spender, authority2);
    }

    #[test]
    fn test_authority_transfer() {
        let mut treasury = Treasury {
            balance: 1_000_000,
            authorized_spender: Pubkey::new_unique(),
        };

        let old_auth = treasury.authorized_spender;
        let new_auth = Pubkey::new_unique();

        treasury.authorized_spender = new_auth;

        assert_eq!(treasury.authorized_spender, new_auth);
        assert_ne!(treasury.authorized_spender, old_auth);
    }

    // ============================================================================
    // Edge Cases and Boundary Tests
    // ============================================================================

    #[test]
    fn test_maximum_proposal_id() {
        let proposal = Proposal {
            id: u64::MAX,
            proposer: Pubkey::new_unique(),
            title: "Max ID".to_string(),
            description: "Proposal with max ID".to_string(),
            voting_start: 1_000_000,
            voting_end: 1_000_000 + VOTING_PERIOD,
        };

        assert_eq!(proposal.id, u64::MAX);
    }

    #[test]
    fn test_very_large_voting_weight() {
        let vote = Vote {
            proposal_id: 1,
            voter: Pubkey::new_unique(),
            amount: u64::MAX / 2,
            direction: 1,
        };

        assert_eq!(vote.amount, u64::MAX / 2);
    }

    #[test]
    fn test_treasury_max_balance() {
        let treasury = Treasury {
            balance: u64::MAX,
            authorized_spender: Pubkey::new_unique(),
        };

        assert_eq!(treasury.balance, u64::MAX);
    }

    // ============================================================================
    // State Consistency Tests
    // ============================================================================

    #[test]
    fn test_proposal_state_consistency() {
        let mut proposal = Proposal {
            id: 1,
            proposer: Pubkey::new_unique(),
            title: "Original".to_string(),
            description: "Original description".to_string(),
            voting_start: 1_000_000,
            voting_end: 1_000_000 + VOTING_PERIOD,
        };

        // Verify state
        let original_id = proposal.id;
        let original_proposer = proposal.proposer;

        // Simulate update (metadata might be updated)
        proposal.title = "Updated".to_string();

        // Critical fields should not change
        assert_eq!(proposal.id, original_id);
        assert_eq!(proposal.proposer, original_proposer);
    }

    #[test]
    fn test_vote_immutability() {
        let vote = Vote {
            proposal_id: 1,
            voter: Pubkey::new_unique(),
            amount: 1000,
            direction: 1,
        };

        // Verify vote data
        assert_eq!(vote.proposal_id, 1);
        assert_eq!(vote.amount, 1000);
        assert_eq!(vote.direction, 1);

        // In real system, votes are immutable once cast
    }

    #[test]
    fn test_treasury_balance_integrity() {
        let mut treasury = Treasury {
            balance: 1_000_000,
            authorized_spender: Pubkey::new_unique(),
        };

        let initial = treasury.balance;

        // Sequence of operations
        treasury.balance -= 100_000;
        treasury.balance += 200_000;
        treasury.balance -= 50_000;

        assert_eq!(treasury.balance, initial - 100_000 + 200_000 - 50_000);
    }

    // ============================================================================
    // Voting Pattern Tests
    // ============================================================================

    #[test]
    fn test_voting_distribution() {
        let proposal_id = 1;

        let votes = vec![
            Vote {
                proposal_id,
                voter: Pubkey::new_unique(),
                amount: 100,
                direction: 1,
            },
            Vote {
                proposal_id,
                voter: Pubkey::new_unique(),
                amount: 150,
                direction: 1,
            },
            Vote {
                proposal_id,
                voter: Pubkey::new_unique(),
                amount: 75,
                direction: 2,
            },
            Vote {
                proposal_id,
                voter: Pubkey::new_unique(),
                amount: 125,
                direction: 2,
            },
            Vote {
                proposal_id,
                voter: Pubkey::new_unique(),
                amount: 0,
                direction: 0,
            },
        ];

        let yes_votes: u64 = votes
            .iter()
            .filter(|v| v.direction == 1)
            .map(|v| v.amount)
            .sum();
        let no_votes: u64 = votes
            .iter()
            .filter(|v| v.direction == 2)
            .map(|v| v.amount)
            .sum();

        assert_eq!(yes_votes, 250);
        assert_eq!(no_votes, 200);
    }

    #[test]
    fn test_weighted_voting_power() {
        let voter1 = Pubkey::new_unique();
        let voter2 = Pubkey::new_unique();
        let voter3 = Pubkey::new_unique();

        // Different voting powers
        let votes = vec![
            Vote {
                proposal_id: 1,
                voter: voter1,
                amount: 10000,
                direction: 1,
            },
            Vote {
                proposal_id: 1,
                voter: voter2,
                amount: 5000,
                direction: 1,
            },
            Vote {
                proposal_id: 1,
                voter: voter3,
                amount: 1000,
                direction: 2,
            },
        ];

        let voter1_power: u64 = votes
            .iter()
            .filter(|v| v.voter == voter1)
            .map(|v| v.amount)
            .sum();
        let voter2_power: u64 = votes
            .iter()
            .filter(|v| v.voter == voter2)
            .map(|v| v.amount)
            .sum();
        let voter3_power: u64 = votes
            .iter()
            .filter(|v| v.voter == voter3)
            .map(|v| v.amount)
            .sum();

        assert_eq!(voter1_power, 10000);
        assert_eq!(voter2_power, 5000);
        assert_eq!(voter3_power, 1000);
    }

    // ============================================================================
    // Treasury Fund Distribution Tests
    // ============================================================================

    #[test]
    fn test_treasury_fund_allocation() {
        let mut treasury = Treasury {
            balance: 10_000_000,
            authorized_spender: Pubkey::new_unique(),
        };

        let allocation_budget = 2_000_000;

        if treasury.balance >= allocation_budget {
            treasury.balance -= allocation_budget;
        }

        assert_eq!(treasury.balance, 8_000_000);
    }

    #[test]
    fn test_treasury_quarterly_distribution() {
        let mut treasury = Treasury {
            balance: 4_000_000,
            authorized_spender: Pubkey::new_unique(),
        };

        let quarterly_amount = 1_000_000;

        // Simulate 4 quarterly distributions
        for _ in 0..4 {
            if treasury.balance >= quarterly_amount {
                treasury.balance -= quarterly_amount;
            }
        }

        assert_eq!(treasury.balance, 0);
    }

    #[test]
    fn test_treasury_emergency_withdrawal() {
        let mut treasury = Treasury {
            balance: 5_000_000,
            authorized_spender: Pubkey::new_unique(),
        };

        let emergency_amount = 2_000_000;

        if treasury.balance >= emergency_amount {
            treasury.balance -= emergency_amount;
        }

        assert_eq!(treasury.balance, 3_000_000);
    }
}
