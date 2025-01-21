test-dev:
	anchor test --skip-build --skip-deploy --provider.cluster devnet

bd:
	anchor build && anchor deploy

bdt:
	make bd && make test-dev