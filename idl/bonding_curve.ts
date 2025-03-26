export type BondingCurve = {
  "version": "0.1.0",
  "name": "bonding_curve",
  "instructions": [
    {
      "name": "updateXyberCoreInstruction",
      "accounts": [
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "newAcceptedBaseMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "InitCoreParams"
          }
        }
      ]
    },
    {
      "name": "mintFullSupplyInstruction",
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "paymentMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "metadataAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenMetadataProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenFactoryProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "External factory program"
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "TokenParams"
          }
        }
      ]
    },
    {
      "name": "buyExactInputInstruction",
      "accounts": [
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "buyer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "paymentMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "buyerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "buyerPaymentAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "baseIn",
          "type": "u64"
        },
        {
          "name": "minAmountOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "buyExactOutputInstruction",
      "accounts": [
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "buyer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "paymentMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "buyerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "buyerPaymentAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "tokensOut",
          "type": "u64"
        },
        {
          "name": "maxPaymentAmount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "sellExactInputInstruction",
      "accounts": [
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "user",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The escrow SPL token account that holds the *payment* tokens (e.g. USDC)."
          ]
        },
        {
          "name": "paymentMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL mint of the payment token (e.g., USDC)."
          ]
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint (fully minted at init)."
          ]
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault that holds project’s tokens."
          ]
        },
        {
          "name": "userTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user’s token account holding tokens."
          ]
        },
        {
          "name": "userPaymentAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user’s associated token account for the *payment* token."
          ]
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "normalizedTokenAmount",
          "type": "u64"
        },
        {
          "name": "minBaseAmountOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "sellExactOutputInstruction",
      "accounts": [
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "user",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The escrow SPL token account that holds the *payment* tokens (e.g. USDC)."
          ]
        },
        {
          "name": "paymentMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL mint of the payment token (e.g., USDC)."
          ]
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint (fully minted at init)."
          ]
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault that holds project’s tokens."
          ]
        },
        {
          "name": "userTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user’s token account holding tokens."
          ]
        },
        {
          "name": "userPaymentAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user’s associated token account for the *payment* token."
          ]
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "lamports",
          "type": "u64"
        },
        {
          "name": "maxTokensIn",
          "type": "u64"
        }
      ]
    },
    {
      "name": "withdrawLiquidity",
      "accounts": [
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "creator",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Escrow token account holding the payment tokens (e.g. USDC)"
          ]
        },
        {
          "name": "baseTokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "adminTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "adminVaultAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "closeXyberCoreInstruction",
      "accounts": [
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "xyberCore",
      "docs": [
        "The sixbte, global state for all tokens."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "publicKey"
          },
          {
            "name": "gradThreshold",
            "type": "u64"
          },
          {
            "name": "bondingCurve",
            "type": {
              "defined": "SmoothBondingCurve"
            }
          },
          {
            "name": "acceptedBaseMint",
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "xyberToken",
      "docs": [
        "One account per unique token. It holds only “token-specific” info."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "isGraduated",
            "type": "bool"
          },
          {
            "name": "mint",
            "type": "publicKey"
          },
          {
            "name": "vault",
            "type": "publicKey"
          },
          {
            "name": "creator",
            "type": "publicKey"
          },
          {
            "name": "totalChains",
            "type": "u8"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "SmoothBondingCurve",
      "docs": [
        "A smooth bonding curve referencing the base asset (e.g., SOL, XBT) deposited.",
        "",
        "Formula: y(x) = A - (K / (C + x))",
        "- A = asymptotic max token supply (in integer \"token units\")",
        "- K = (token * lamport), controlling how quickly we approach A",
        "- C = virtual pool offset (in base_tokens)",
        "",
        "NOTE: We no longer store `x_total_base_deposit` inside the struct.",
        "Instead, the caller passes the current x each time."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "aTotalTokens",
            "docs": [
              "Asymptotic total token supply (in \"raw\" tokens)"
            ],
            "type": "u64"
          },
          {
            "name": "kVirtualPoolOffset",
            "docs": [
              "Controls how quickly we approach A (token * lamport)"
            ],
            "type": "u128"
          },
          {
            "name": "cBondingScaleFactor",
            "docs": [
              "Virtual pool offset (in base_tokens)"
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "InitCoreParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": {
              "option": "publicKey"
            }
          },
          {
            "name": "gradThreshold",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "bondingCurve",
            "type": {
              "option": {
                "defined": "SmoothBondingCurve"
              }
            }
          },
          {
            "name": "acceptedBaseMint",
            "type": {
              "option": "publicKey"
            }
          }
        ]
      }
    },
    {
      "name": "TokenParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "symbol",
            "type": "string"
          },
          {
            "name": "uri",
            "type": "string"
          },
          {
            "name": "totalChains",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "XyberInstructionType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "BuyExactIn"
          },
          {
            "name": "BuyExactOut"
          },
          {
            "name": "SellExactIn"
          },
          {
            "name": "SellExactOut"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "GraduationTriggered",
      "fields": [
        {
          "name": "buyer",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "escrowBalance",
          "type": "u64",
          "index": false
        },
        {
          "name": "vault",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "creator",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "escrow",
          "type": "publicKey",
          "index": false
        }
      ]
    },
    {
      "name": "XyberSwapEvent",
      "fields": [
        {
          "name": "ixType",
          "type": {
            "defined": "XyberInstructionType"
          },
          "index": false
        },
        {
          "name": "tokenSeed",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "user",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "baseAmount",
          "type": "u64",
          "index": false
        },
        {
          "name": "tokenAmount",
          "type": "u64",
          "index": false
        },
        {
          "name": "vaultTokenAmount",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "InsufficientTokenSupply",
      "msg": "Custom error: Token supply is not enough to fulfill buy request"
    },
    {
      "code": 6001,
      "name": "MathOverflow",
      "msg": "Provide a smaller amount. Use normalized tokens (e.g., raw value / 10 ** decimals)."
    },
    {
      "code": 6002,
      "name": "Unauthorized",
      "msg": "Unauthorized: Caller is not authorized to perform this action."
    },
    {
      "code": 6003,
      "name": "BondingCurveNotGraduated",
      "msg": "Bonding Curve not graduated: pool has not reached the required threshold."
    },
    {
      "code": 6004,
      "name": "InsufficientTokenVaultBalance",
      "msg": "Insufficient token balance in the vault to fulfill the request."
    },
    {
      "code": 6005,
      "name": "InsufficientEscrowBalance",
      "msg": "Insufficient escrow balance: Not enough tokens in escrow to complete the operation."
    },
    {
      "code": 6006,
      "name": "TokenIsGraduated",
      "msg": "Token has graduated: The bonding curve is no longer active as the token is now listed on a DEX."
    },
    {
      "code": 6007,
      "name": "InvalidSeed",
      "msg": "Invalid seed: the provided seed must be exactly 32 bytes in length."
    },
    {
      "code": 6008,
      "name": "SlippageExceeded",
      "msg": "Slippage exceeded user-defined limit."
    },
    {
      "code": 6009,
      "name": "WrongPaymentMint",
      "msg": "Wrong payment mint provided."
    }
  ]
};

export const IDL: BondingCurve = {
  "version": "0.1.0",
  "name": "bonding_curve",
  "instructions": [
    {
      "name": "updateXyberCoreInstruction",
      "accounts": [
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "newAcceptedBaseMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "InitCoreParams"
          }
        }
      ]
    },
    {
      "name": "mintFullSupplyInstruction",
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "paymentMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "metadataAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenMetadataProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenFactoryProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "External factory program"
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "TokenParams"
          }
        }
      ]
    },
    {
      "name": "buyExactInputInstruction",
      "accounts": [
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "buyer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "paymentMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "buyerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "buyerPaymentAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "baseIn",
          "type": "u64"
        },
        {
          "name": "minAmountOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "buyExactOutputInstruction",
      "accounts": [
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "buyer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "paymentMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "buyerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "buyerPaymentAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "tokensOut",
          "type": "u64"
        },
        {
          "name": "maxPaymentAmount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "sellExactInputInstruction",
      "accounts": [
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "user",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The escrow SPL token account that holds the *payment* tokens (e.g. USDC)."
          ]
        },
        {
          "name": "paymentMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL mint of the payment token (e.g., USDC)."
          ]
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint (fully minted at init)."
          ]
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault that holds project’s tokens."
          ]
        },
        {
          "name": "userTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user’s token account holding tokens."
          ]
        },
        {
          "name": "userPaymentAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user’s associated token account for the *payment* token."
          ]
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "normalizedTokenAmount",
          "type": "u64"
        },
        {
          "name": "minBaseAmountOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "sellExactOutputInstruction",
      "accounts": [
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "user",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The escrow SPL token account that holds the *payment* tokens (e.g. USDC)."
          ]
        },
        {
          "name": "paymentMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL mint of the payment token (e.g., USDC)."
          ]
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint (fully minted at init)."
          ]
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault that holds project’s tokens."
          ]
        },
        {
          "name": "userTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user’s token account holding tokens."
          ]
        },
        {
          "name": "userPaymentAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user’s associated token account for the *payment* token."
          ]
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "lamports",
          "type": "u64"
        },
        {
          "name": "maxTokensIn",
          "type": "u64"
        }
      ]
    },
    {
      "name": "withdrawLiquidity",
      "accounts": [
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "xyberToken",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "mint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenSeed",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "creator",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Escrow token account holding the payment tokens (e.g. USDC)"
          ]
        },
        {
          "name": "baseTokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "adminTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "adminVaultAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "closeXyberCoreInstruction",
      "accounts": [
        {
          "name": "xyberCore",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "xyberCore",
      "docs": [
        "The sixbte, global state for all tokens."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "publicKey"
          },
          {
            "name": "gradThreshold",
            "type": "u64"
          },
          {
            "name": "bondingCurve",
            "type": {
              "defined": "SmoothBondingCurve"
            }
          },
          {
            "name": "acceptedBaseMint",
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "xyberToken",
      "docs": [
        "One account per unique token. It holds only “token-specific” info."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "isGraduated",
            "type": "bool"
          },
          {
            "name": "mint",
            "type": "publicKey"
          },
          {
            "name": "vault",
            "type": "publicKey"
          },
          {
            "name": "creator",
            "type": "publicKey"
          },
          {
            "name": "totalChains",
            "type": "u8"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "SmoothBondingCurve",
      "docs": [
        "A smooth bonding curve referencing the base asset (e.g., SOL, XBT) deposited.",
        "",
        "Formula: y(x) = A - (K / (C + x))",
        "- A = asymptotic max token supply (in integer \"token units\")",
        "- K = (token * lamport), controlling how quickly we approach A",
        "- C = virtual pool offset (in base_tokens)",
        "",
        "NOTE: We no longer store `x_total_base_deposit` inside the struct.",
        "Instead, the caller passes the current x each time."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "aTotalTokens",
            "docs": [
              "Asymptotic total token supply (in \"raw\" tokens)"
            ],
            "type": "u64"
          },
          {
            "name": "kVirtualPoolOffset",
            "docs": [
              "Controls how quickly we approach A (token * lamport)"
            ],
            "type": "u128"
          },
          {
            "name": "cBondingScaleFactor",
            "docs": [
              "Virtual pool offset (in base_tokens)"
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "InitCoreParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": {
              "option": "publicKey"
            }
          },
          {
            "name": "gradThreshold",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "bondingCurve",
            "type": {
              "option": {
                "defined": "SmoothBondingCurve"
              }
            }
          },
          {
            "name": "acceptedBaseMint",
            "type": {
              "option": "publicKey"
            }
          }
        ]
      }
    },
    {
      "name": "TokenParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "symbol",
            "type": "string"
          },
          {
            "name": "uri",
            "type": "string"
          },
          {
            "name": "totalChains",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "XyberInstructionType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "BuyExactIn"
          },
          {
            "name": "BuyExactOut"
          },
          {
            "name": "SellExactIn"
          },
          {
            "name": "SellExactOut"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "GraduationTriggered",
      "fields": [
        {
          "name": "buyer",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "escrowBalance",
          "type": "u64",
          "index": false
        },
        {
          "name": "vault",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "creator",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "escrow",
          "type": "publicKey",
          "index": false
        }
      ]
    },
    {
      "name": "XyberSwapEvent",
      "fields": [
        {
          "name": "ixType",
          "type": {
            "defined": "XyberInstructionType"
          },
          "index": false
        },
        {
          "name": "tokenSeed",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "user",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "baseAmount",
          "type": "u64",
          "index": false
        },
        {
          "name": "tokenAmount",
          "type": "u64",
          "index": false
        },
        {
          "name": "vaultTokenAmount",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "InsufficientTokenSupply",
      "msg": "Custom error: Token supply is not enough to fulfill buy request"
    },
    {
      "code": 6001,
      "name": "MathOverflow",
      "msg": "Provide a smaller amount. Use normalized tokens (e.g., raw value / 10 ** decimals)."
    },
    {
      "code": 6002,
      "name": "Unauthorized",
      "msg": "Unauthorized: Caller is not authorized to perform this action."
    },
    {
      "code": 6003,
      "name": "BondingCurveNotGraduated",
      "msg": "Bonding Curve not graduated: pool has not reached the required threshold."
    },
    {
      "code": 6004,
      "name": "InsufficientTokenVaultBalance",
      "msg": "Insufficient token balance in the vault to fulfill the request."
    },
    {
      "code": 6005,
      "name": "InsufficientEscrowBalance",
      "msg": "Insufficient escrow balance: Not enough tokens in escrow to complete the operation."
    },
    {
      "code": 6006,
      "name": "TokenIsGraduated",
      "msg": "Token has graduated: The bonding curve is no longer active as the token is now listed on a DEX."
    },
    {
      "code": 6007,
      "name": "InvalidSeed",
      "msg": "Invalid seed: the provided seed must be exactly 32 bytes in length."
    },
    {
      "code": 6008,
      "name": "SlippageExceeded",
      "msg": "Slippage exceeded user-defined limit."
    },
    {
      "code": 6009,
      "name": "WrongPaymentMint",
      "msg": "Wrong payment mint provided."
    }
  ]
};
