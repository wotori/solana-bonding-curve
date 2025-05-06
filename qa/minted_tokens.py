import base58
from solana.rpc.api import Client, Pubkey

client = Client("https://api.devnet.solana.com")

PROGRAM_ID = Pubkey(base58.b58decode("HL1jyNFAJa8EhuuqpZJfLLTsXsfk1yCGMX8XpGssrxQQ"))
print("Program ID:", PROGRAM_ID)

resp = client.get_program_accounts(PROGRAM_ID, encoding="base64")
if not resp.value:
    print("No program accounts found or RPC error:", resp)
    exit()

accounts = resp.value
print(f"Total Program Accounts: {len(accounts)}\n")

for acc_info in accounts:
    raw_data = acc_info.account.data  # already raw bytes

    if len(raw_data) == 118:
        # XyberCore layout (118 bytes):
        #   0..8    = discriminator (8 bytes)
        #   8..40   = admin (Pubkey, 32 bytes)
        #   40..42  = grad_threshold (u16, 2 bytes)
        #   42..82  = SmoothBondingCurve (40 bytes total)
        #       - 42..50  = a_total_tokens (u64, 8 bytes)
        #       - 50..66  = k_virtual_pool_offset (u128, 16 bytes)
        #       - 66..74  = c_bonding_scale_factor (u64, 8 bytes)
        #       - 74..82  = x_total_base_deposit (u64, 8 bytes)
        #   82..114 = accepted_base_mint (Pubkey, 32 bytes)
        #   114..118= graduate_dollars_amount (u32, 4 bytes)
        #
        admin_bytes              = raw_data[8:40]
        grad_threshold_bytes     = raw_data[40:42]
        a_total_tokens_bytes     = raw_data[42:50]
        k_virtual_pool_bytes     = raw_data[50:66]
        c_bonding_scale_bytes    = raw_data[66:74]
        x_total_base_bytes       = raw_data[74:82]
        accepted_mint_bytes      = raw_data[82:114]
        grad_dollars_bytes       = raw_data[114:118]

        admin_str     = base58.b58encode(admin_bytes).decode("utf-8")
        grad_threshold = int.from_bytes(grad_threshold_bytes, "little")

        a_total_tokens   = int.from_bytes(a_total_tokens_bytes, "little")
        k_virtual_offset = int.from_bytes(k_virtual_pool_bytes, "little")   # u128 => int
        c_bonding_scale  = int.from_bytes(c_bonding_scale_bytes, "little")
        x_total_base     = int.from_bytes(x_total_base_bytes, "little")

        accepted_str  = base58.b58encode(accepted_mint_bytes).decode("utf-8")
        grad_dollars  = int.from_bytes(grad_dollars_bytes, "little")  # u32

        print(f"=== XyberCore (118 bytes) Account: {acc_info.pubkey} ===")
        print(f"  admin:                {admin_str}")
        print(f"  grad_threshold:       {grad_threshold}")
        print("  -- bonding_curve --")
        print(f"    a_total_tokens:       {a_total_tokens}")
        print(f"    k_virtual_pool_offset:{k_virtual_offset}")
        print(f"    c_bonding_scale_factor: {c_bonding_scale}")
        print(f"    x_total_base_deposit:   {x_total_base}")
        print(f"  accepted_base_mint:   {accepted_str}")
        print(f"  graduate_dollars_amount: {grad_dollars}\n")
        
    else:
        # Anchor discriminator = raw_data[0..8]
        # is_graduated (1 byte) at [8]
        is_graduated_byte = raw_data[8]

        # mint (Pubkey, 32 bytes) at [9..41]
        mint_bytes = raw_data[9:41]

        # vault (Pubkey, 32 bytes) at [41..73]
        vault_bytes = raw_data[41:73]

        # creator (Pubkey, 32 bytes) at [73..105]
        creator_bytes = raw_data[73:105]

        # Convert fields
        is_graduated = (is_graduated_byte != 0)
        mint_str = base58.b58encode(mint_bytes).decode("utf-8")
        vault_str = base58.b58encode(vault_bytes).decode("utf-8")
        creator_str = base58.b58encode(creator_bytes).decode("utf-8")

        if '111' in mint_str:
            continue
        
        # Print only if there's a valid creator
        if creator_str:
            print(f"--- PDA Account: {acc_info.pubkey} ---")
            print(f"is_graduated = {is_graduated}")
            print(f"mint         = {mint_str}")
            print(f"creator      = {creator_str}")
            print(f"raw_data_len = {len(raw_data)}")
            print()
            
print(f"Total: {len(accounts)}")