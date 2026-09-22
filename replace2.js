const fs = require('fs');

const files = [
  'packages/dashboard/src/components/OraclePrices.tsx',
  'packages/dashboard/src/components/marketing/TerminalShowcase.tsx',
  'packages/dashboard/src/components/ActionsPanel.tsx',
  'packages/dashboard/src/app/docs/security/page.tsx',
  'packages/dashboard/src/app/docs/sdk/page.tsx',
  'packages/dashboard/src/app/docs/quickstart/page.tsx',
  'packages/dashboard/src/app/docs/page.tsx',
  'packages/dashboard/src/app/docs/cli/page.tsx',
  'packages/dashboard/src/app/app/rehearse/page.tsx',
  'packages/dashboard/src/app/app/forensics/page.tsx',
  'packages/dashboard/src/lib/pyth.ts',
  'packages/dashboard/src/components/docs/MirrorPipeline.tsx',
  'packages/dashboard/src/components/marketing/MirrorVisual.tsx'
];

for (const file of files) {
  if (!fs.existsSync(file)) continue;
  let c = fs.readFileSync(file, 'utf8');
  const orig = c;
  
  // Replace Helius in UI text and terminal commands
  c = c.replace(/HELIUS_RPC/g, 'ALCHEMY_RPC');
  c = c.replace(/helius-rpc\.com/g, 'alchemy.com');
  c = c.replace(/Helius/g, 'Alchemy');
  c = c.replace(/helius\.dev/g, 'alchemy.com');
  c = c.replace(/Triton/g, 'Infura');
  
  // Terminal commands and UI references to ecosystem tools
  c = c.replace(/--preload pyth raydium/g, '--preload oracle dex');
  c = c.replace(/\"pyth\", \"raydium\", \"orca\", \"marinade\"/g, '\"oracle\", \"dex\", \"amm\", \"staking\"');
  
  if (file.includes('TerminalShowcase.tsx')) {
    c = c.replace(/HELIUS/g, 'ALCHEMY');
  }

  // Double check any remaining rendered Pyth
  c = c.replace(/Oracle · Pyth/g, 'Oracle · Market Data');
  c = c.replace(/Pyth · SOL\/USD/g, 'Market Data · ETH/USD');
  c = c.replace(/Pyth accounts/g, 'Market Data accounts');

  if (c !== orig) {
    fs.writeFileSync(file, c, 'utf8');
    console.log('Updated', file);
  }
}
