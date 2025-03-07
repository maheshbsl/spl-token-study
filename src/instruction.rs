pub enum TokenInstruction<'a> {
    InitializeMint {
        // number of base 10 digits to the right of the decimal place.
        decimals: u8,
        // the authority to mint tokens
        mint_authority: Pubkey,
        // the freeze authority of the mint
        freeze_authority: COption<Pubkey>,
    },
    InitializeAccount,
    InitializeMultisig {
        // the number of signers `m` required to validate this
        // multisignature account
        m: u8,
    },
    Transfer {
        // the amount of tokens to transfer
        amount: u64,
    },
    Approve {
        // the amount of tokens the delegate is allowed to transfer
        amount: u64,
    },
    Revoke,
    SetAuthority {
        // the type of authority to update
        authority_type: AuthorityType,
        // the new authority
        new_authority: COption<Pubkey>,
    },
    MintTo {
        // the amount of new tokens to mint
        amount: u64,
    },
    Burn {
        //the amount of tokens to burn
        amount: u64,
    },
    CloseAccount,
    FreezeAccount,
    ThawAccount,
    TransferChecked {
        // the amount of tokens to transfer
        amount: u64,
        // the number of base 10 digits to the right of the decimal place.
        decimals: u8,
    },
    ApproveChecked {
        // the amount of tokens the delegate is allowed for
        amount: u64,
        // expected number of base 10 digits to the right of the decimal place
        decimals: u8,
    },
    MintToChecked {
        // the amount of new tokens to mint
        amount: u64,
        // expected number of base 10 digits to the right of the decimal place
        decimals: u8,
    },
    BurnChecked {
        // the amount of tokens to burn
        amount: u64,
        // expected number of base 10 digits to the right of the decimal place
        decimals: u8,
    },
    InitializeAccount2 {
        // the new account's owner/ multisignature
        owner: Pubkey,
    },
    SyncNative,
    InitializeAccount3 {
        // the new account's owner/multisignature
        owner: Pubkey,
    },
    InitializeMultisig2 {
        // the number of signers `m` required to validate this
        // multisignature account
        m: u8,
    },
    InitializeMint2 {
        // number of base 10 digits to the right of the decimal place.
        decimals: u8,
        // the authority/multisignature to mint tokens
        mint_authority: Pubkey,
        // the freeze authority/multisignature of the mint
        freeze_authority: COption<Pubkey>,
    },
    GetAccountDataSize,
    InitializeImmutableOwner,
    AmountToUiAmount {
        // the amount of tokens to reformat
        amount: u64,
    },
    UiAmountToAmount {
        // the `ui_amount` of tokens to reformat
        ui_amount: &'a str,
    },
}

impl<'a> TokenInstruction<'a> {
    // unpacks a byte buffer into a `TokenInstruction`
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use TokenError::InvalidInstruction;

        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                // extract the decimals, mint authority, and freeze authority
                let (&decimals, rest) = rest.split_first().ok_or(InvalidInstruction)?;
                let (mint_authority, rest) = Self::unpack_pubkey(rest)?;
                let (freeze_authority, _rest) = Self::unpack_pubkey_option(rest)?;
                Self::InitializeMint {
                    decimals,
                    mint_authority,
                    freeze_authority,
                }
            }
            1 => Self::InitializeAccount,
            2 => {
                let &m = rest.first().ok_or(InvalidInstruction)?;
                Self::InitializeMultisig { m }
            }
            3 | 4 | 7 | 8 => {
                // extract the amount (8bytes) and convert to u64
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                match tag {
                    3 => Self::Transfer { amount },
                    4 => Self::Approve { amount },
                    7 => Self::MintTo { amount },
                    8 => Self::Burn { amount },
                    _ => unreachable!(),
                }
            }
            5 => Self::Revoke,
            6 => {
                // extract the authority tye (1 byte)
                let (authority_type, rest) = rest
                    .split_first()
                    .ok_or_else(|| ProgramError::from(InvalidInstruction))
                    // convert the byte to an authority type
                    .and_then(|(&t, rest)| Ok((AuthorityType::from(t)?, rest)))?;
                let (new_authority, _rest) = Self::unpack_pubkey_option(rest)?;

                Self::SetAuthority {
                    authority_type,
                    new_authority,
                }
            }
            9 => Self::CloseAccount,
            10 => Self::FreezeAccount,
            11 => Self::ThawAccount,
            12 => {
                // extract the amount (8 bytes) and convert to u64
                let (amount, decimals, _rest) = Self::unpack_amount_decimals(rest)?;
                Self::TransferChecked { amount, decimals }
            }
            13 => {
                let (amount, decimals, _rest) = Self::unpack_amount_decimals(rest)?;
                Self::ApproveChecked { amount, decimals }
            }
            14 => {
                let (amount, decimals, _rest) = Self::unpack_amount_decimals(rest)?;
                Self::MintToChecked { amount, decimals }
            }
            15 => {
                let (amount, decimals, _rest) = Self::unpack_amount_decimals(rest)?;
                Self::BurnChecked { amount, decimals }
            }
            16 => {
                // extract owner (32 bytes) and convert to pubkey
                let (owner, _rest) = Self::unpack_pubkey(rest)?;
                Self::InitializeAccount2 { owner }
            }
            17 => Self::SyncNative,
            18 => {
                let (owner, rest) = Self::unpack_pubkey(rest)?;
                Self::InitializeAccount3 { owner }
            }
            19 => {
                // extract the number of signers `m` (1 byte)
                let &m = rest.first().ok_or(InvalidInstruction)?;
                Self::InitializeMultisig2 { m }
            }
            20 => {
                // extract the decimals
                let (&decimals, rest) = rest.split_first().ok_or(InvalidInstruction)?;
                // extract the mint authority (32 bytes) and convert to pubkey
                let (mint_authority, rest) = Self::unpack_pubkey(rest)?;
                // extract the freeze authority (optional 32 bytes) and convert to pubkey
                let (freeze_authority, _rest) = Self::unpack_pubkey_option(rest)?;

                Self::InitializeMint2 {
                    decimals,
                    mint_authority,
                    freeze_authority,
                }
            }
            21 => Self::GetAccountDataSize,
            22 => Self::InitializeImmutableOwner,
            23 => {
                // extract amount 8 bytes and convert to u64
                let (amount, _rest) = Self::unpack_amount(rest)?;
                Self::AmountToUiAmount { amount }
            }
            24 => {
                let ui_amount = std::str::from_utf8(rest).map_err(|_| InvalidInstruction)?;
                Self::UiAmountToAmount { ui_amount }
            }
            _ => return Err(TokenError::InvalidInstruction.into()),
        })
    }

    // packs a `TokenInstruction` into a byte buffer
    pub fn pack(&self) -> Vec<u8> {
        // create a buffer with the size of the instruction
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::InitializeMint {
                decimals,
                ref mint_authority,
                ref freeze_authority,
            } => {
                buf.push(0);
                buf.push(decimals);
                buf.extend_from_slice(mint_authority.as_ref());
                Self::pack_pubkey_option(freeze_authority, &mut buf);
            }
            Self::InitializeAccount => buf.push(1),
            &Self::InitializeMultisig { m } => {
                buf.push(2);
                buf.push(m);
            }
            &Self::Transfer { amount } => {
                buf.push(3);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            &Self::Approve { amount } => {
                buf.push(4);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            &Self::MintTo { amount } => {
                buf.push(7);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            &Self::Burn { amount } => {
                buf.push(8);
                buff.extend_from_slice(&amount.to_le_bytes());
            }
            &Self::Revoke => buf.push(5),
            &Self::SetAuthority {
                authority_type,
                ref new_authority,
            } => {
                buf.push(6);
                buf.push(authority_type.into());
                Self::pack_pubkey_option(new_authority, &mut buf);
            }
            &Self::CloseAccount => buf.push(9),
            &Self::FreezeAccount => buf.push(10),
            &Self::ThawAccount => buf.push(11),

            &Self::TransferChecked { amount, decimals } => {
                buf.push(12);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.push(decimals);
            }
            &Self::ApproveChecked { amount, decimals } => {
                buf.push(13);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.push(decimals);
            }
            &Self::MintToChecked { amount, decimals } => {
                buf.push(14);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.push(decimals);
            }
            &Self::BurnChecked { amount, decimals } => {
                buf.push(15);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.push(decimals);
            }
            &Self::InitializeAccount2 { owner } => {
                buf.push(16);
                buf.extend_from_slice(owner.as_ref());
            }
            &Self::SyncNative => buf.push(17),
            &Self::InitializeAccount3 { owner } => {
                buf.push(18);
                buf.extend_from_slice(owner.as_ref());
            }
            &Self::InitializeMultisig2 { m } => {
                buf.push(19);
                buf.push(m);
            }
            &Self::InitializeMint2 {
                decimals,
                ref mint_authority,
                ref freeze_authority,
            } => {
                buf.push(20);
                buf.push(decimals);
                buf.extend_from_slice(mint_authority.as_ref());
                Self::pack_pubkey_option(freeze_authority, &mut buf);
            }
            &Self::GetAccountDataSize => buf.push(21),
            &Self::InitializeImmutableOwner => buf.push(22),
            &Self::AmountToUiAmount { amount } => {
                buf.push(23);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            &Self::UiAmountToAmount { ui_amount } => {
                buf.push(24);
                buf.extend_from_slice(ui_amount.as_bytes());
            }
            _ => unreachable!(),
        };
        buf
    }

    // unpacks a pubkey from a byte slice
    fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        // check if the input is at least 32 bytes
        if input.len() >= 32 {
            // first 32 bytes are the pubkey and rest will be returned
            let (key, rest) = input.split_at(32);
            // convert the first 32 bytes to a pubkey
            let pubkey = Pubkey::new_from(key).map_err(|| TokenError::InvalidInstruction)?;
            Ok(pubkey, rest)
        } else {
            Err(TokenError::InvalidInstruction.into())
        }
    }

    fn unpack_pubkey_option(input: &[u8]) -> Result<(COption<Pubkey>, &[u8]), ProgramError> {
        // extract the first byte to determine if the pubkey is present
        match input.split_first() {
            // if the first byte is 0, the pubkey is not present
            Option::Some(&0, rest) => Ok((COption::None, rest)),
            // if the first byte is 1 and there are at least 32 bytes remaining, extract the pubkey
            Option::Some(&1, rest) if rest.len() >= 32 => {
                let (key, rest) = rest.split_at(32);
                let pubkey = Pubkey::new_from(key).map_err(|| TokenError::InvalidInstruction)?;
                Ok((COption::Some(pubkey), rest))
            }
            _ => Err(TokenError::InvalidInstruction.into()),
        }
    }

    fn pack_pubkey_optioin(value: &COption<Pubkey>, buf: &mut Vec<u8>) {
        match *value {
            COption::Some(ref key) => {
                buf.push(1);
                buf.extend_from_slice(&key.to_bytes());
            }
            COption::None => buf.push(0),
        }
    }

    fn unpack_64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        let value = input
            // U64_BYTES is a constant which is 8
            // get the first 8 bytes of the input
            .get(..U64_BYTES)
            // try_into() converts a slice into a [u8; 8] array
            // ok() returns an option Some([u8; 8]) if the conversion is successful
            .and_then(|slice| slice.try_into().ok())
            // map() converts the [u8; 8] array into a u64
            .map(u64::from_le_bytes)
            // ok_or() returns an error if the conversion is not successful
            .ok_or(TokenError::InvalidInstruction)?;
        // return the value if successful and remaining bytes are U64_BYTES
        Ok((value, &input[u64_BYTES..]))
    }

    fn unpack_amount_decimals(input: &[u8]) -> Result<(u64, u8, &[u8]), ProgramError> {
        let (amount, rest) = Self::unpack_b4(input)?;
        let (&decimals, rest) = rest.split_first().ok_or(TokenError::InvalidInstruction)?;
        Ok((amount, decimals, rest))
    }
}

// Specifies the authority type for `SetAuthority` instructions
pub enum AuthorityType {
    // Authority to mint new tokens
    MintTokens,
    // Authority to freeze account associated with the Mint
    FreezeAccount,
    // Owner of a given account
    AccountOwner,
    // Authority to close a token account
    CloseAccount,
}

impl AuthorityType {
    fn into(&self) -> u8 {
        match self {
            AuthorityType::MintTokens => 0,
            AuthorityType::FreezeAccount => 1,
            AuthorityType::AccountOwner => 2,
            AuthorityType::CloseAccount => 3,
        }
    }

    fn from(index: u8) -> Result<Self, ProgramError> {
        match index {
            0 => Ok(AuthorityType::MintTokens),
            1 => Ok(AuthorityType::FreezeAccount),
            2 => Ok(AuthorityType::AccountOwner),
            3 => Ok(AuthorityType::CloseAccount),
            _ => Err(ProgramError::InvalidArgument.into()),
        }
    }
}

// Creates a InitializeMint instruction

pub fn initialize_mint(
    // the program id of the token program (where the instruction is sent)
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
    freeze_authority_pubkey: Option<&Pubkey>,
    decimals: u8,
) -> Result<Instruction, ProgramError> {
    // check program validity
    // this ensures that the provided `token_program_is` is a valid SPL Token program
    check_program_account(token_program_id)?;
    // convert freeze authority to COption
    let freeze_authority = freeze_authority_pubkey.cloned().into();

    // create the token instruction
    let data = TokenInstruction::InitializeMint {
        decimals,
        mint_authority: mint_authority_pubkey,
        freeze_authority,
    }
    .pack();

    // define account Metadata
    let accounts = vec![
        AccountMeta::new(*mint_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    })
}

// Creates a `InitializeMint2` instruction

pub fn initialize_mint2(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
    freeze_authority_pubkey: Option<&Pubkey>,
    decimals: u8,
) -> Result<Instruction, ProgramError> {
    // check program validity
    check_program_account(token_program_id)?;

    // convert freeze authority to COption
    let freeze_authority = freeze_authority_pubkey.cloned().into();

    // creat the token instruction
    let data = TokenInstruction::InitializeMint2 {
        decimals,
        mint_authority: *mint_authority_pubkey,
        freeze_authority,
    }
    .pack();

    // define account metadata
    let accounts = vec![AccountMeta::new(*mint_pubkey, false)];

    Ok(Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    })
}

// Creats a `InitializeAccount` instruction

pub fn initialize_account(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    // check the validity of the program
    check_program_account(token_program_id)?;

    let data = TokenInstruction::InitializeAccount.pack();

    let accounts = vec![
        AccountMeta::new(*account_pubkey, false),
        AccountMeta::new_readonly(*mint_pubkey, false),
        AccountMeta::new_readonly(*owner_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    })
}

pub fn initialize_account2(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    let data = TokenInstruction::InitializeAccount2 {
        owner: *owner_pubkey,
    }
    .pack();

    let accounts = vec![
        AccoutMeta::new(*account_pubkey, false),
        AccountMeta::new_readonly(*mint_pubkey, false),
    ];

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

pub fn initialize_account3(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    // ensure that the provided token_program id is official spl token id
    check_program_account(token_program_id)?;

    // serialize the data
    let data = TokenInstruction::InitializeAccount3 {
        owner: *owner_pubkey,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new(*account_pubkey, false),
        AccountMeta::new_readonly(*mint_pubkey, false),
    ];

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

// Creates a InitializeMultisig instruction
pub fn initialize_multisig(
    token_program_id: &Pubkey,
    multisig_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    m: u8,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    if !is_valid_signer_index(m as usize)
        || !is_valid_signer_index(signer_pubkeys.len())
        || m as usize > signer_pubkeys.len()
    {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let data = TokenInstruction::InitializeMultisig { m }.pack();

    let mut accounts = Vec::with_capacity(1 + 1 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*multisig_pubkey, false));
    accounts.push(AccoutnMeta::new_readonly(sysvar::rent::id(), false));
    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, false));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

// Creates a `InitializeMultisig2` instruction

pub fn initialize_multisig2(
    token_program_id: &Pubkey,
    multisig_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    m: u8,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    if !is_valid_signer_index(m as usize)
        || !is_valid_signer_index(signer_pubkeys.len())
        || m as usize > signer_pubkeys.len()
    {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let data = TokenInstruction::InitializeMultisig2 { m }.pack();

    let accounts = Vec::with_capacity(1 + 1 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*multisig_pubkey, false));
    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, false));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

// Creates a `Transfer` instruction

pub fn transfer(
    token_program_id: &Pubkey,   // token program address
    source_pubkey: &Pubkey,      // source token account address (sender pubkey)
    destination_pubkey: &Pubkey, // destination token account address (receiver pubkey)
    authority_pubkey: &Pubkey,   // authority who can approve the transfer
    signer_pubkeys: &[&Pubkey],  // optional multiple signers (for multisig )
    amount: u64,                 // the amount of tokens to transfer
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;
    let data = TokenInstruction::Transfer { amount }.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*source_pubkey, false));
    accounts.push(AccountMeta::new(*destination_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *authority_pubkey,
        signer_pubkeys.is_empty(),
    ));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

/// Creates a `Revoke` Instruction

pub fn revoke(
    token_program_id: &Pubkey,
    source_pubkey: &Pubkey, // the pubkey of the account the delegate is revoked.
    owner_pubkey: &Pubkey,  // owner of the token account
    signer_pubkeys: &[&Pubkey],
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;
    let data = TokenInstruction::Revoke.pack();

    let accounts = Vec::with_capacity(2 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*source_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *owner_pubkey,
        signer_pubkeys.empty(),
    ));

    for signer_pubkey in signer.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true))
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

/// Creates a `SetAutority` instruction

pub fn set_authority(
    token_program_id: &Pubkey,             //pubkey of token program
    owned_pubkey: &Pubkey, // the pubkey of the account whose authority is being updated
    new_authority_pubkey: Option<&Pubkey>, //the new authority
    authority_type: AuthorityType, // authority type (mint, ...)
    owner_pubkey: &Pubkey, // current authority
    signer_pubkeys: &[&Pubkey],
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;
    let new_authority = new_authority_pubkey.cloned().into();
    let data = TokenInstruction::SetAuthority {
        new_authority,
        authority_type,
    }
    .pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*owned_pubkey, false));
    accounts.push(AccountMeta::new_readonly(*owner, signer_pubkeys.is_empty()));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

/// Creates a `MintTo` instruction
pub fn mint_to(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    account_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    let data = TokenInstruction::MintTo { amount }.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*mint_pubkey, false));
    account.push(AccountMeta::new(*account_pubkey, false));
    account.push(AccountMeta::new_readonly(
        *owner_pubkey,
        signer_pubkeys.is_empty(),
    ));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

/// Creates a `Burn` instruction
pub fn burn(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    let data = TokenInstruction::Burn { amount }.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*account_pubkey, false));
    accounts.push(AccountMeta::new(*mint_pubkey, false));
    account.push(AccountMeta::new_readonly(
        *authority_pubkey,
        signer_pubkeys.is_empty(),
    ));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

/// Creates a `CloseAccount` instruction
pub fn close_account(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
) -> Result<Instruction, ProgramError> {

    check_program_account(token_program_id)?;

    let data = TokenInstruction::CloseAccount.pack();

    let accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    account.push(AccountMeta::new(*account_pubkey, false));
    account.push(AccountMeta::new(*destination_pubkey, false));
    account.push(AccountMeta::new_readonly(
        *owner_pubkey,
        signer_pubkeys.is_empty(),
    ));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(*signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_progam_id,
        data,
        accounts,
    })
}

/// Creates a `FreezeAccount` instruction
pub fn freeze_account(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    signer_pubkeys: &Pubkey,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;
    
    let data = TokenInstruction::FreezeAccount.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    account.push(AccountMeta::new(*account_pubkey, false));
    account.push(AccountMeta::new_readonly(*mint_pubkey, false));
    account.push(AccountMeta::new_readonly(
        *owner_pubkey,
        signer_pubkeys.is_empty(),
    ));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

/// Creates a `ThawAccount` instructioin

pub fn thaw_account(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    let data = TokenInstruction::ThawAccount.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*account_pubkey, false));
    accounts.push(AccountMeta::new_readonly(*mint_pubkey, false));
    accounts.push(AccountMeat::new_readonly(
        *owner_pubkey,
        signer_pubkeys.is_empty(),
    ));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

/// Creates a `TransferChecked` instruction
pub fn transfer_checked(
    token_program_id: &Pubkey,
    source_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
    decimals: u8,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    let data = TokenInstruction::TransferChecked {
        amount, decimals
    }
    .pack();

    let mut accounts = Vec::with_capacity(4 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*source_pubkey, false)),
    accounts.push(AccountMeta::new(*destination_pubkey, false));
    accounts.push(AccountMeta::new_readonly(*mint_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *authority_pubkey,
        signer_pubkeys.is_empty(),
    ));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: token_program_id,
        data,
        accounts,
    })
}

/// Creates a `ApproveChecked` instruction
pub fn approve_checked(
    token_program_id: &Pubkey,
    source_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    delegate_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
    decimals: u8,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    let data = TokenInstruction::ApproveChecked {
        amount,
        decimals,
    }
    .pack();

    let mut accounts = Vec::with_capacity(4 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*source_pubkey, false));
    accounts.push(AccountMeta::new_readonly(*mint_pubkey, false));
    accounts.push(AccountMeta::new(*deligate_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *owner_pubkey,
        signer_pubkeys.is_empty(),
    ));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

/// Creates a `MintToChecked` instruction
pub fn mint_to_checked(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    account_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
    decimals: u8,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    let data = TokenInstruction::MintToChecked { amount, decimals }.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*mint_pubkey, false));
    accounts.push(AccountMeta::new(*account_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *owner_pubkey,
        signer_pubkeys.is_empty(),
    ));
    
    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

/// Creates a `BurnChecked` instruction
pub fn burn_checked(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
    decimals: u8,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    let data = TokenInstruction::BurnChecked { amount, decimals }.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*account_pubkey, false));
    accounts.push(AccountMeta::new(*mint_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *authority_pubkey,
        signer_pubkeys.is_empty(),
    ));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        data,
        accounts,
    })
}

/// Creates a `sysn-native` instruction
pub fn sync_native(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id);

    Ok(Instruction {
        program_id: *token_progam_id,
        accounts: vec![AccountMeta::new(*account_pubkey, false)],
        data: TokenInstruction::SyncNative.pack(),
    })
}

/// Creates a `GetAccountDataSize` instruction
pub fn get_account_data_size(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    Ok(Instruction {
        program_id: *token_progam_id,
        accounts: vec![AccountMeta::new_readonly(*account_pubkey, false)],
        data: TokenInstruction::GetAccountDataSize.pack(),
    })
}

/// Creates a `InitializeImmutableOwner` instruction
pub fn initialize_immutable_owner(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_progam_id)?;

    Ok(Instruction {
        program_id: *token_progam_id,
        accounts: vec![AccountMeta::new(*account_pubkey, false)],
        data: TokenInstruction::InitializeImmutableOwner.pack(),
    })
}

/// Creates an `AmountToUiAmount` instruction
pub fn amount_to_ui_amount(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    Ok(Instruction {
        program_id: *token_program_id,
        accounts: vec![AccountMeta::new_readonly(*mint_pubkey, false)],
        data: TokenInstruction::AmountToUiAmount { amount }.pack(),
    })
}

/// Creates a `UiAmountToAmount` instruction
pub fn ui_amount_to_amount(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    ui_amount: &str,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    Ok(Instruction {
        program_id: *token_program_id,
        accounts: vec![AccountMeta::new_readonly(*mint_pubkey, false)],
        data: TokenInstruction::UiAmountToAmount { ui_amount }.pack(),
    })
}

/// Utility function that checks index is between `MIN_SIGNERS` and
/// `MAX_SIGNERS`
pub fn is_valid_signer_index(index: usize) -> bool {
    (MIN_SIGNERS..=MAX_SIGNERS).contains(&index)
}