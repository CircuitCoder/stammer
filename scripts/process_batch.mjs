import fs from 'fs';

const lines = fs.readFileSync(process.argv[2]).toString('utf-8').split('\n').filter(e => !!e);

const input = fs.createWriteStream(process.argv[3]);
const std = fs.createWriteStream(process.argv[4]);

let isOdd = true;
for(const line of lines) {
  (isOdd ? input : std).write(line.toLowerCase() + '\n');
  isOdd = !isOdd;
}
