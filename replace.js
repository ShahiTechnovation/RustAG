const fs = require('fs');
const files = [
  'packages/dashboard/src/app/(marketing)/early-access/page.tsx',
  'packages/dashboard/src/app/api/early-access/route.ts',
  'packages/dashboard/src/app/app/forensics/page.tsx',
  'packages/dashboard/src/app/app/page.tsx',
  'packages/dashboard/src/app/app/rehearse/page.tsx',
  'packages/dashboard/src/app/docs/architecture/page.tsx',
  'packages/dashboard/src/app/docs/cli/page.tsx',
  'packages/dashboard/src/app/docs/concepts/page.tsx',
  'packages/dashboard/src/app/docs/layout.tsx',
  'packages/dashboard/src/app/docs/page.tsx',
  'packages/dashboard/src/app/docs/quickstart/page.tsx',
  'packages/dashboard/src/app/docs/sdk/page.tsx',
  'packages/dashboard/src/app/docs/security/page.tsx',
  'packages/dashboard/src/app/layout.tsx',
  'packages/dashboard/src/components/Footer.tsx',
  'packages/dashboard/src/components/LogoMark.tsx',
  'packages/dashboard/src/components/marketing/FeatureBento.tsx',
  'packages/dashboard/src/components/marketing/Hero.tsx',
  'packages/dashboard/src/components/marketing/HowItWorks.tsx',
  'packages/dashboard/src/components/marketing/TerminalShowcase.tsx'
];

for (const file of files) {
  if (!fs.existsSync(file)) continue;
  let c = fs.readFileSync(file, 'utf8');
  const orig = c;
  
  // Safe replacements for text that appears in user-facing JSX/strings.
  c = c.replace(/Solana transaction/g, 'Robinhood Chain transaction');
  c = c.replace(/Solana program/g, 'smart contract');
  c = c.replace(/Solana mainnet/g, 'Robinhood Chain');
  c = c.replace(/Solana ecosystem/g, 'Robinhood Chain ecosystem');
  c = c.replace(/for Solana/g, 'for Robinhood Chain');
  c = c.replace(/on Solana/g, 'on Robinhood Chain');
  c = c.replace(/Solana/g, 'Robinhood Chain');
  c = c.replace(/solana/g, 'robinhood chain');
  
  // Squads -> multisig
  c = c.replace(/Squads v4/g, 'multisig');
  c = c.replace(/Squads UI/g, 'multisig UI');
  c = c.replace(/a Squads/g, 'a multisig');
  c = c.replace(/Squads proposal/g, 'multisig proposal');
  c = c.replace(/Squads/g, 'multisig');
  
  // SVM -> EVM (only in words, careful not to break variable names like LiteSVM unless intended)
  // Let's replace LiteSVM with EVM sandbox
  c = c.replace(/LiteSVM instance/g, 'EVM sandbox instance');
  c = c.replace(/LiteSVM sandbox/g, 'EVM sandbox');
  c = c.replace(/LiteSVM/g, 'EVM sandbox');
  c = c.replace(/\bSVM\b/g, 'EVM');
  
  // SOL -> ETH
  c = c.replace(/\bSOL\b/g, 'ETH');
  
  // Undo specific replacements that broke imports/variables if any
  c = c.replace(/@robinhood chain\/web3\.js/g, '@solana/web3.js');
  c = c.replace(/robinhood chain-rpc-client/g, 'solana-rpc-client');
  c = c.replace(/multisigDecoder/g, 'SquadsDecoder');
  
  // Make sure to not break links like solana.com
  c = c.replace(/robinhood chain\.com/g, 'solana.com');

  if (c !== orig) {
    fs.writeFileSync(file, c, 'utf8');
    console.log('Updated', file);
  }
}
