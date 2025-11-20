
use std::collections::HashSet;

use solana_pubkey::Pubkey;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionDomain {
    // Vote Execution Domain (VED) - Domain 1
    // Contains only Vote Program
    Vote,
    
    // User Execution Domain (UED) - Domain 0  
    // Contains all other programs and accounts
    User,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    CrossDomainAccess {
        transaction_signature: String,
        attempted_domains: Vec<ExecutionDomain>,
    },
    
    UnauthorizedDomainAccess {
        account: Pubkey,
        requested_domain: ExecutionDomain,
        actual_domain: ExecutionDomain,
    },
    
    InvalidDomainTransition {
        account: Pubkey,
        reason: String,
    },
}

// Account to domain mapping
#[allow(dead_code)]
pub struct DomainRegistry {
    // Accounts currently in the Vote Domain
    vote_domain_accounts: HashSet<Pubkey>,
    
    // Accounts scheduled for domain transition next epoch
    pending_transitions: Vec<DomainTransition>,
    
    current_epoch: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DomainTransition {
    pub account: Pubkey,
    pub from_domain: ExecutionDomain,
    pub to_domain: ExecutionDomain,
    pub scheduled_epoch: u64,
}

#[allow(dead_code)]
impl DomainRegistry {
    pub fn new() -> Self {
        Self {
            vote_domain_accounts: HashSet::new(),
            pending_transitions: Vec::new(),
            current_epoch: 0,
        }
    }
    
    pub fn get_account_domain(&self, pubkey: &Pubkey) -> ExecutionDomain {
        if self.vote_domain_accounts.contains(pubkey) {
            ExecutionDomain::Vote
        } else {
            ExecutionDomain::User
        }
    }
    
    pub fn is_vote_account(&self, pubkey: &Pubkey, owner: &Pubkey) -> bool {
        owner == &solana_vote_program::id() || 
        self.vote_domain_accounts.contains(pubkey)
    }

    pub fn add_to_vote_domain(&mut self, pubkey: Pubkey) {
        self.vote_domain_accounts.insert(pubkey);
    }
    
    pub fn schedule_domain_transition(
        &mut self,
        account: Pubkey,
        from: ExecutionDomain,
        to: ExecutionDomain,
    ) -> Result<(), DomainError> {
        // Can only transition once per epoch
        if self.pending_transitions.iter().any(|t| t.account == account) {
            return Err(DomainError::InvalidDomainTransition {
                account,
                reason: "Transition already scheduled for this epoch".to_string(),
            });
        }
        
        self.pending_transitions.push(DomainTransition {
            account,
            from_domain: from,
            to_domain: to,
            scheduled_epoch: self.current_epoch + 1,
        });
        
        Ok(())
    }
    
    pub fn apply_epoch_transitions(&mut self, new_epoch: u64) {
        self.current_epoch = new_epoch;
        
        self.pending_transitions.retain(|transition| {
            if transition.scheduled_epoch == new_epoch {
                match transition.to_domain {
                    ExecutionDomain::Vote => {
                        self.vote_domain_accounts.insert(transition.account);
                    }
                    ExecutionDomain::User => {
                        self.vote_domain_accounts.remove(&transition.account);
                    }
                }
                false 
            } else {
                true 
            }
        });
    }
}
