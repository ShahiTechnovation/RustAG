const fs = require('fs');
const path = require('path');

function findFiles(dir) {
  let results = [];
  const list = fs.readdirSync(dir);
  for (let file of list) {
    file = path.resolve(dir, file);
    const stat = fs.statSync(file);
    if (stat && stat.isDirectory()) {
      results = results.concat(findFiles(file));
    } else if (file.endsWith('.tsx') || file.endsWith('.ts')) {
      results.push(file);
    }
  }
  return results;
}

const files = findFiles('packages/dashboard/src');

for (const file of files) {
  let c = fs.readFileSync(file, 'utf8');
  const orig = c;
  
  c = c.replace(/PRELOAD REAL MAINNET STATE FROM THE PROTOCOLS YOU BUILD AGAINST/g, 'BUILT AROUND THE ROBINHOOD CHAIN ECOSYSTEM');
  c = c.replace(/Live Pyth/gi, 'Live Market Data');
  
  // Replace LogoTicker array
  c = c.replace(
    /const PROTOCOLS = \["Jupiter", "Pyth", "Raydium", "Orca", "Marinade", "SPL Token", "Anchor", "Helius"\];/g,
    'const PROTOCOLS = ["UNISWAP", "PLEIADES", "CHAINLINK", "ALCHEMY", "ALLIUM", "LAYERZERO", "BITGO", "TRM LABS", "BLOCKSCOUT", "QUICKNODE"];'
  );

  // General Replacements
  // Doing it safely so we don't break JS code syntax like imports or vars. 
  // We'll replace it in text and some UI strings.
  
  c = c.replace(/\bPyth\b/g, 'Oracle');
  c = c.replace(/\bRaydium\b/g, 'Uniswap');
  c = c.replace(/\bOrca\b/g, 'AMM');
  c = c.replace(/\bMarinade\b/g, 'Staking');
  c = c.replace(/\bSPL Token\b/g, 'ERC20 Token');
  c = c.replace(/\bSPL-token\b/g, 'ERC20');
  c = c.replace(/\bspl-token\b/g, 'erc20');
  c = c.replace(/\bAnchor\b/g, 'Foundry');
  c = c.replace(/\bHelius\b/g, 'Alchemy');
  c = c.replace(/\bJupiter\b/g, 'Uniswap');
  c = c.replace(/\bSolscan\b/g, 'Blockscout');
  c = c.replace(/\bPhantom\b/g, 'MetaMask');
  c = c.replace(/\bMetaplex\b/g, 'NFT');
  c = c.replace(/\bJito\b/g, 'MEV');
  c = c.replace(/SOL\/USD/g, 'ETH/USD');
  c = c.replace(/\bSOL\b/g, 'ETH');

  if (c !== orig) {
    fs.writeFileSync(file, c, 'utf8');
    console.log('Updated', file);
  }
}
