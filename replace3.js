const fs = require('fs');
const file = 'packages/dashboard/src/components/ActionsPanel.tsx';
let c = fs.readFileSync(file, 'utf8');
c = c.replace(/\"jupiter\"/g, '\"router\"');
fs.writeFileSync(file, c, 'utf8');
