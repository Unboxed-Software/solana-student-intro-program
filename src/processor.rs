use crate::error::StudentIntroError;
use crate::instruction::IntroInstruction;
use crate::state::{Reply, ReplyCounter, StudentInfo};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use std::convert::TryInto;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = IntroInstruction::unpack(instruction_data)?;
    match instruction {
        IntroInstruction::InitUserInput { name, message } => {
            add_student_intro(program_id, accounts, name, message)
        }
        IntroInstruction::UpdateStudentIntro { name, message } => {
            update_student_intro(program_id, accounts, name, message)
        }
        IntroInstruction::AddReply { reply } => add_reply(program_id, accounts, reply),
    }
}

pub fn add_student_intro(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    message: String,
) -> ProgramResult {
    msg!("Adding student intro...");
    msg!("Name: {}", name);
    msg!("Message: {}", message);
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;
    let reply_counter = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref()], program_id);
    if pda != *user_account.key {
        msg!("Invalid seeds for PDA");
        return Err(StudentIntroError::InvalidPDA.into());
    }
    let studentinfo_discriminator = "studentinfo";
    let total_len: usize =
        (4 + studentinfo_discriminator.len()) + 1 + (4 + name.len()) + (4 + message.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(StudentIntroError::InvalidDataLength.into());
    }
    let account_len: usize = 1000;

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            user_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            user_account.clone(),
            system_program.clone(),
        ],
        &[&[initializer.key.as_ref(), &[bump_seed]]],
    )?;

    msg!("PDA created: {}", pda);

    msg!("unpacking state account");
    let mut account_data =
        try_from_slice_unchecked::<StudentInfo>(&user_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    msg!("checking if studentinfo account is already initialized");
    if account_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.name = name;
    account_data.msg = message;
    account_data.is_initialized = true;
    msg!("serializing account");
    account_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    msg!("create reply counter");
    let counter_discriminator = "counter";
    let counter_len: usize = (4 + counter_discriminator.len()) + 1 + 1;

    let rent = Rent::get()?;
    let counter_rent_lamports = rent.minimum_balance(counter_len);

    let (counter, counter_bump) =
        Pubkey::find_program_address(&[pda.as_ref(), "reply".as_ref()], program_id);
    if counter != *reply_counter.key {
        msg!("Invalid seeds for PDA");
        return Err(ProgramError::InvalidArgument);
    }

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            reply_counter.key,
            counter_rent_lamports,
            counter_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            reply_counter.clone(),
            system_program.clone(),
        ],
        &[&[pda.as_ref(), "reply".as_ref(), &[counter_bump]]],
    )?;
    msg!("reply counter created");

    let mut counter_data =
        try_from_slice_unchecked::<ReplyCounter>(&reply_counter.data.borrow()).unwrap();

    msg!("checking if counter account is already initialized");
    if counter_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    counter_data.discriminator = counter_discriminator.to_string();
    counter_data.counter = 0;
    counter_data.is_initialized = true;
    msg!("reply count: {}", counter_data.counter);
    counter_data.serialize(&mut &mut reply_counter.data.borrow_mut()[..])?;

    Ok(())
}

pub fn update_student_intro(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    message: String,
) -> ProgramResult {
    msg!("Updating student intro...");
    msg!("Name: {}", name);
    msg!("Message: {}", message);
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;

    msg!("unpacking state account");
    let mut account_data =
        try_from_slice_unchecked::<StudentInfo>(&user_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    msg!("checking if movie account is initialized");
    if !account_data.is_initialized() {
        msg!("Account is not initialized");
        return Err(StudentIntroError::UninitializedAccount.into());
    }

    if user_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    let (pda, _bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref()], program_id);
    if pda != *user_account.key {
        msg!("Invalid seeds for PDA");
        return Err(StudentIntroError::InvalidPDA.into());
    }
    let update_len: usize = 1 + (4 + account_data.name.len()) + (4 + message.len());
    if update_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(StudentIntroError::InvalidDataLength.into());
    }

    account_data.name = account_data.name;
    account_data.msg = message;
    msg!("serializing account");
    account_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    Ok(())
}

pub fn add_reply(program_id: &Pubkey, accounts: &[AccountInfo], reply: String) -> ProgramResult {
    msg!("Adding Reply...");
    msg!("Reply: {}", reply);

    let account_info_iter = &mut accounts.iter();

    let replier = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;
    let reply_counter = next_account_info(account_info_iter)?;
    let reply_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let mut counter_data =
        try_from_slice_unchecked::<ReplyCounter>(&reply_counter.data.borrow()).unwrap();

    let reply_discriminator = "reply";
    let account_len: usize = (4 + reply_discriminator.len()) + 1 + 32 + (4 + reply.len());

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[
            user_account.key.as_ref(),
            counter_data.counter.to_be_bytes().as_ref(),
        ],
        program_id,
    );
    if pda != *reply_account.key {
        msg!("Invalid seeds for PDA");
        return Err(StudentIntroError::InvalidPDA.into());
    }

    invoke_signed(
        &system_instruction::create_account(
            replier.key,
            reply_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            replier.clone(),
            reply_account.clone(),
            system_program.clone(),
        ],
        &[&[
            user_account.key.as_ref(),
            counter_data.counter.to_be_bytes().as_ref(),
            &[bump_seed],
        ]],
    )?;

    msg!("Created Reply Account");
    let mut reply_data = try_from_slice_unchecked::<Reply>(&reply_account.data.borrow()).unwrap();

    msg!("checking if comment account is already initialized");
    if reply_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    reply_data.discriminator = reply_discriminator.to_string();
    reply_data.studentinfo = *user_account.key;
    reply_data.reply = reply;
    reply_data.is_initialized = true;
    reply_data.serialize(&mut &mut reply_account.data.borrow_mut()[..])?;
    msg!("Reply Count: {}", counter_data.counter);
    counter_data.counter += 1;
    counter_data.serialize(&mut &mut reply_counter.data.borrow_mut()[..])?;
    Ok(())
}
