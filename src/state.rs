use solana_program:: {
    program_pack::{IsInitialized, Pack, Sealed },
    program_error::ProgramError,
    pubkey::Pubkey,
};

use spl_token::state::COption;  
use arrayref::{array_ref, array_refs, mut_array_refs, mut_array_ref, array_mut_ref};  


pub struct Mint {
    /// Owner authority who can mint new tokens
    pub mint_authority: COption<Pubkey>,
    /// Total supply of Tokens
    pub supply: u64,
    /// Number of base 10 digits to the right of decimal place
    pub decimals: u8,
    /// Is token Initialized
    pub is_initialized: bool,
    /// Optional authority to freeze token accounts
    pub freeze_authority: COption<Pubkey>,
}

impl Sealed for Mint {}
impl IsInitialized for Mint {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Mint {
    const LEN: usize = 82;
    /// This function deserialize a byte slice `src` into a Mint struct
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        // extract a fixed 82-bytes slice form input
        let src = array_ref![src, 0, 82];
        /// Splits the byte slice into:
        ///  mint_authority (36 bytes)
        ///  supply (8 bytes)
        ///  decimals (1 byte)
        ///  is_initialized (1 byte)
        ///  freeze_authority (36 bytes)

        let (mint_authority, supply, decimals, is_initialized, freeze_authority) =
            array_ref![src, 36, 8, 1, 1, 36];
        
        // unpack fields
        /// unpack_coption_key converts 36-byte field into Option<Pubkey>
        /// Some(Pubkey) if exit
        /// None if empty
        let mint_authority = unpack_coption_key(mint_authority)?;

        /// converts 8 bytes into a u64 integer using little-endian order
        let supply = u64::from_le_bytes(*supply);

        /// read the single byte decimal value
        let decimals = decimals[0];

        /// Converts is_initialized bytes value into boolean
        /// 1 -> true
        /// 0  -> false
        /// any other value error
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        let freeze_authority = unpack_coption_key(freeze_authority)?;

        /// Return the Mint struct
        Ok(Mint {
            mint_authority,
            supply,
            decimals,
            is_initialized,
            freeze_authority,
        })
    }

    /// This function serialize a Mint struct into a mutuable byte slice(dst)
    fn pack_into_slice(&self, dst: &mut [u8]) {
        // `array_mut_ref` is a macro that converts the mutable
        // slice `dst` into a fixed-sized array reference of 82 bytes
        let dst = array_mut_ref![dst, 0, 82];

        // `mut_array_ref` splits the 82 bytes array into five mutable references
        let (
            mint_authority_dst, // 36 bytes for the mint authority(optional pubkey)
            supply_dst,  // 8 bytes for token supply (u64)
            decimals_dst, // 1 byte for the number of decimals
            is_initialized_dst, // 1 byte for wheter the mint is initialized
            freeze_authority_dst, // 36 bytes for the optional freeze authority
        ) = mut_array_ref![dst, 36, 8, 1, 1, 36];

        // Destructure the `Mint` instance into its fields for easier access
        let &Mint {
            ref mint_authority,
            supply,
            decimals,
            is_initialized,
            ref freeze_authority,
        } = self;

        // Pack the `mint_authority` into the corresponding 36-byte slice
        pack_coption_key(mint_authority, mint_authority_dst);
        // converts supply (a u64 value) to little_endian value and stores it
        *supply_dst = supply.to_le_bytes();
        // assigns the 1 byte decimals value
        decimals_dst[0] = decimals;
        // boolean is converted to u8
        // false -> 0
        // true -> 1
        is_initialized_dst[0] = is_initialized as u8;
        pack_coption_key(freeze_authority, freeze_authority_dst);
    }
}


pub struct Account {
    /// The mint associated with this account
    pub mint: Pubkey,
    /// The owner of this Account
    pub owner: Pubkey,
    /// Amount of tokens this account holds
    pub amount: u64,
    /// Authority delegated to transfer tokens
    pub delegate: COption<Pubkey>,
    /// State of the account
    pub state: AccountState,
    /// Is this a native token
    pub is_native: COption<u64>,
    /// Delegated amount
    pub delegated_amount: u64,
    /// Close Authority
    pub close_authority: COption<Pubkey>,
}

impl Account {
    /// Checks if account is frozen
    pub fn is_frozen(&self) -> bool {
        self.state == AccountState::Frozen
    }

    /// Checks if account is native
    pub fn is_native(&self) -> bool {
        self.is_native().is_some()
    }
}

impl Sealed for Account {}
impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.state != AccountState::Uninitialized
    }
}


// Implement `Pack` trait for `Account` struct
impl Pack for Account {
    const LEN: usize = 165;
    /// This function deserialize a byte slice `src` into a Result<Account, ProgramError>
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 165];
        let (mint, owner, amount, delegate, state, is_native, delegated_amount, close_autority) =
            array_refs![src, 32, 32, 8, 36, 1, 12, 8, 36];

        Ok(Account{
            mint: Pubkey::new_from_array(*mint),
            owner: Pubkey::new_from_array(*owner),
            amount: u64::from_le_bytes(*amount),
            delegate: unpack_coption_key(delegate)?,
            state: AccountState::try_from_primitive(state[0])
                .or(Err(ProgramError::InvalidAccountData))?,
            is_native: unpack_coption_u64(is_native)?,
            delegated_amount: u64::from_le_bytes(*delegated_amount),
            close_authority: unpack_coption_key(close_autority)?,
        })
    }

    // this function serialize a `Account` struct into a mutable slice
    fn pack_pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 165];
        let (
            mint_dst,
            owner_dst,
            amount_dst,
            delegate_dst,
            state_dst,
            is_native_dst,
            delegated_amount_dst,
            close_authority_dst,
        ) = mut_array_refs![dst, 32, 32, 8, 36, 1, 12, 8, 36];

        let &Account {
            ref mint,
            ref owner,
            amount,
            ref delegate,
            state,
            ref is_native,
            delegated_amount,
            ref close_authority,

        } = self;
        mint_dst.copy_from_slice(mint.as_ref());
        owner_dst.copy_from_slice(owner.as_ref());
        *amount_dst = amount.to_le_bytes();
        pack_coption_key(delegate, delegate_dst);
        state_dst[0] = state as u8;
        pack_coption_u64(is_native, is_native_dst);
        *delegated_amount_dst = delegated_amount.to_le_bytes();
        pack_coption_key(close_authority, close_authority_dst);
    }
}

pub enum AccountState {
    /// Account is not yet initialized
    Uninitialized,
    /// Account is initialized: the account owner or delegate may
    /// perform permitted operations on this account
    Initialized,
    /// Account has been frozen by the mint freeze authority. Neither the 
    /// account owner nor the delegate are able to perform operations 
    /// on this account
    Frozen,
}

pub struct Multisig {
    /// Number of signers required
    pub m: u8,
    /// Numbr of valid signers
    pub n: u8,
    /// Is initialized
    pub is_initialized: bool,
    /// Signer public keys
    pub signers: [Pubkey; MAX_SIGNERS],
}


/// Helpers

fn pack_coption_key(src: &COption<Pubkey>, dst: &mut [u8; 36]) {
    /// Takes a reference to a `COption<Pubkey>` which may contain either `Some(Pubkey)` or `None`
    /// `dst`: A mutable byte array of size 36 where data will be packed
    
    /// Splits the 36-byte slice into:
    /// * `tag` - 4 bytes that store whether the value exists (1) or not (0)
    /// * `body` - 32 bytes that store the `Pubkey` if it exists
    let (tag, body) = mut_array_refs![dst, 4, 32];

    match src {
        COption::Some(key) => {
            *tag = [1, 0, 0, 0]; // 4-byte little-endian representaion of 1
            body.copy_from_slice(key.as_ref()); // Copy 32 bytes of Pubkey 
        }
        COption::None => {
            *tag = [0; 4];
        }
    }

}

/// deserialize a 36-byte slice into a COption<Pubkey> 
fn unpack_coption_key(src: &[u8; 36]) ->Result<COption<Pubkey>, ProgramError> {
    /// Take a reference to a 36-byte array containing the serialized COption<Pubkey>.
    let (tag , body) = array_refs![src, 4, 32];

    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(Pubkey::new_from_array(*body))),
        _ => Err(ProgramError::InvalidAccountData),
    }
}

/// serialize a COption<u64> into a 12 byte slice
fn pack_coption_u64(src: &COption<u64>, dst: &mut [u8; 12]) {
    let (tag, body) = mut_array_refs![dst, 4, 8];
    match src {
        COption::Some(amount) => {
            *tag = [1, 0, 0, 0];
            body = amount.to_le_bytes();
        }
        COption::None => {
            *tag = [0; 4];
        }
    }
}

/// deserialize a 12-byte slice to a COption<u64>
fn unpack_coption_u64(src: &[u8; 12]) -> Result<COption<u64>, ProgramError> {
    let (tag, body) = array_refs![src, 8, 4];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(u64::from_be_bytes(*body))),
        _ => Err(ProgramError::InvalidAccountData),
    }
}
