test:
	anchor test --skip-build --skip-deploy -- --verbose --provider.cluster devnet

bd:
	anchor build && anchor deploy

bdtest:
	make bd && make t