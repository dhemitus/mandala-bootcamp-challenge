use crate::staking::StakingConfig;
use crate::system::SystemConfig;
use std::collections::HashMap;

pub trait GovernanceConfig: StakingConfig + SystemConfig {}

pub struct Proposal {
    description: String,
    yes_votes: u32,
    no_votes: u32,
    status: ProposalStatus,
}

#[derive(Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Approved,
    Rejected,
}

pub struct GovernancePallet<T: GovernanceConfig> {
    pub proposals: HashMap<u32, Proposal>,
    pub votes: HashMap<(T::AccountId, u32), bool>, // (voter, proposal_id) -> vote_type
    next_proposal_id: u32,
}

impl<T: GovernanceConfig> GovernancePallet<T> {
    pub fn new() -> Self {
        Self { 
            proposals: HashMap::new(),
            votes: HashMap::new(),
            next_proposal_id: 0
        }
    }

    // Create a new proposal
    pub fn create_proposal(
        &mut self,
        _creator: T::AccountId,
        description: String,
    ) -> Result<u32, &'static str> {
        self.next_proposal_id += 1;

        self.proposals.insert(self.next_proposal_id, Proposal{
            description,
            yes_votes: 0,
            no_votes: 0,
            status: ProposalStatus::Active
        });

        Ok(self.next_proposal_id)
    }

    // Vote on a proposal (true = yes, false = no)
    pub fn vote(
        &mut self,
        voter: T::AccountId,
        proposal_id: u32,
        vote_type: bool,
    ) -> Result<(), &'static str> {
        if self.votes.contains_key(&(voter.clone(), proposal_id)) {
            return Err("can not vote more than once");
        }

        if !self.proposals.contains_key(&proposal_id) {
            return Err("proposal does not exist");
        }

        self.votes.insert((voter, proposal_id), vote_type);
        let proposal = self.get_proposal(proposal_id).expect("could not found the proposal");

        if proposal.status != ProposalStatus::Active {
            return  Err("proposal was Finalized");
        }
        let yes_votes = if vote_type {proposal.yes_votes + 1} else {proposal.yes_votes};
        let no_votes = if !vote_type  {proposal.no_votes + 1} else {proposal.no_votes};

        let new_proposal = Proposal {
            description: proposal.description.clone(),
            yes_votes,
            no_votes,
            status: proposal.status.clone(),
        };

        self.proposals.insert(proposal_id, new_proposal);

        Ok(())
    }

    // Get proposal details
    pub fn get_proposal(&self, proposal_id: u32) -> Option<&Proposal> {
        self.proposals.get(&proposal_id)
    }

    // Finalize a proposal (changes status based on votes)
    pub fn finalize_proposal(&mut self, proposal_id: u32) -> Result<ProposalStatus, &'static str> {
        let proposal = self.get_proposal(proposal_id).expect("could not found the proposal");

        let yes_votes = proposal.yes_votes;
        let no_votes = proposal.no_votes;

        let status = if yes_votes > no_votes {
            ProposalStatus::Approved
        } else {
            ProposalStatus::Rejected
        };

        let new_proposal = Proposal {
            description: proposal.description.clone(),
            yes_votes,
            no_votes,
            status: status.clone(),
        };

        self.proposals.insert(proposal_id, new_proposal);

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;

    #[test]
    fn test_governance_should_work() {
        let alice = 1u64;
        let bob = 2u64;
        let charlie = 3u64;

        let mut governance = GovernancePallet::<Runtime>::new();

        // Create a proposal
        let proposal_id = governance
            .create_proposal(alice, "Increase validator rewards".to_string())
            .unwrap();

        // Cast votes
        governance.vote(alice, proposal_id, true).unwrap(); // Yes vote
        governance.vote(bob, proposal_id, true).unwrap(); // Yes vote
        governance.vote(charlie, proposal_id, false).unwrap(); // No vote

        // Check proposal status before finalization
        let proposal = governance.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.yes_votes, 2);
        assert_eq!(proposal.no_votes, 1);

        // Finalize proposal
        let status = governance.finalize_proposal(proposal_id).unwrap();
        assert!(matches!(status, ProposalStatus::Approved));

        // Check proposal is now approved
        let finalized_proposal = governance.get_proposal(proposal_id).unwrap();
        assert!(matches!(
            finalized_proposal.status,
            ProposalStatus::Approved
        ));
    }
}
