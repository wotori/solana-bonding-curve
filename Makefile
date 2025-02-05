test:
	anchor test --skip-build --skip-deploy -- --verbose --provider.cluster devnet

deploy:
	anchor build && anchor deploy --program-name bonding_curve --program-keypair target-deploy-keypair.json

bd:
	anchor build && anchor deploy

bdtest:
	make bd && make t