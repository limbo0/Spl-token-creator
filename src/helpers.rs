use anyhow::Result;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};

// while minting, need to create ATA for receiving wallet.
pub fn mint_token(
    client: &RpcClient,
    owner: &Keypair,
    amount: u64,
    mint_public_key: pubkey::Pubkey,
    mint_to: &pubkey::Pubkey,
) -> Result<()> {
    let mint_to_inst = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint_public_key,
        &mint_to,
        &owner.pubkey(),
        &[&owner.pubkey()],
        amount,
    )?;

    let tx = Transaction::new_signed_with_payer(
        &[mint_to_inst],
        Some(&owner.pubkey()),
        &[owner],
        client.get_latest_blockhash()?,
    );
    client.send_and_confirm_transaction(&tx)?;
    println!(
        "minted: {:?} {:?} tokens to {:?} address sucessfully!",
        amount, mint_public_key, mint_to
    );

    Ok(())
}

// Creating token and updating metadata has to be two seperate transactions.
fn create_token(
    client: &RpcClient,
    payer: &Keypair,
    token_account: &Keypair,
    space: u64,
) -> Result<()> {
    // check for minimun to store in solana blockchain.
    let rent = client.get_minimum_balance_for_rent_exemption(space.try_into()?)?;
    println!("minimun_rent_needed: {:?}", rent);

    // create account and assign owner as spl token program.
    let create_account_inst = system_instruction::create_account(
        &payer.pubkey(),
        &token_account.pubkey(),
        rent,
        space,
        &spl_token::ID,
    );

    // init the above created account as mint account.
    // this account is responsible for storing the global information of a token.
    let initialize_mint_inst = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &token_account.pubkey(),
        &payer.pubkey(),
        Some(&payer.pubkey()),
        6,
    )?;

    // create a associated token account(pda) for the mint account.
    // this accounts store the relationship between a wallet and mint account.
    let create_ata_inst =
        spl_associated_token_account::instruction::create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &token_account.pubkey(),
            &spl_token::ID,
        );

    let associated_token_account =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &payer.pubkey(),
            &token_account.pubkey(),
            &spl_token::ID,
        );

    println!("ATA: {:?}", associated_token_account);

    // mint tokens to associated_token_account created above.
    let inst_mint_to = spl_token::instruction::mint_to(
        &spl_token::ID,
        &token_account.pubkey(),
        &associated_token_account,
        &payer.pubkey(),
        &[&token_account.pubkey(), &payer.pubkey()],
        100000000,
    )?;

    let blockhash = client.get_latest_blockhash()?;
    println!("blockhash: {:?}", blockhash);

    let tx = Transaction::new_signed_with_payer(
        &[
            create_account_inst,
            initialize_mint_inst,
            create_ata_inst,
            inst_mint_to,
        ],
        Some(&payer.pubkey()),
        &[payer, token_account],
        blockhash,
    );

    client.send_and_confirm_transaction(&tx)?;

    println!("creation of token succeed!");

    Ok(())
}

// Creating token and updating metadata has to be two seperate transactions.
fn update_metadata(
    client: &RpcClient,
    payer: &Keypair,
    token_account: pubkey::Pubkey,
) -> Result<()> {
    println!("Updating metadata!");
    // creating metadata pda.
    let (expected_pda, bump_seed) = solana_program::pubkey::Pubkey::find_program_address(
        &[
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            token_account.as_ref(),
        ],
        &mpl_token_metadata::ID,
    );

    let metadata_pda =
        mpl_token_metadata::accounts::Metadata::create_pda(token_account, bump_seed)?;

    println!(
        "metadataPda: {:?} expected_pda: {:?}",
        metadata_pda, expected_pda
    );

    // attach metadata to the token
    let metadata_v3_args = mpl_token_metadata::instructions::CreateMetadataAccountV3InstructionArgs {
        data: mpl_token_metadata::types::DataV2 {
            name: "cr1tikal".to_string(),
            symbol: "cr1tikal".to_string(),
            uri: "https://blue-controversial-marlin-879.mypinata.cloud/ipfs/QmePNpXMof22GkUxmXXRd8SDg67nxdNGwaAX8QC88Jm6V8".to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        is_mutable: true,
        collection_details: None,
    };

    let metadata_v3 = mpl_token_metadata::instructions::CreateMetadataAccountV3 {
        metadata: metadata_pda,
        mint: token_account,
        mint_authority: payer.pubkey(),
        payer: payer.pubkey(),
        update_authority: (payer.pubkey(), true),
        system_program: system_program::ID,
        rent: Some(payer.pubkey()),
    };

    let create_metadata_inst = metadata_v3.instruction(metadata_v3_args);

    let blockhash = client.get_latest_blockhash()?;
    println!("blockhash: {:?}", blockhash);

    let tx = Transaction::new_signed_with_payer(
        &[create_metadata_inst],
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );

    client.send_and_confirm_transaction(&tx)?;

    println!("Updated metadata sucessfully!");

    Ok(())
}
