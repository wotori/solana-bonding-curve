test:
	anchor test --skip-build --skip-deploy -- --verbose --provider.cluster devnet

deploy:
	anchor build && anchor deploy --program-name bonding_curve --program-keypair target-deploy-keypair.json

bd:
	anchor build && anchor deploy

bdtest:
	make bd && make t

qa-xyber-core-stats:
	npx mocha -r ts-node/register tests/xybercore-stats.spec.ts --timeout 100000