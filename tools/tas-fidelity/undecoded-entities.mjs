// Which vanilla `Solid`/`Trigger` class names appear in the vendored maps, and are they decoded?
import fs from 'node:fs';
import path from 'node:path';

const mapsDir = process.argv[2] ?? 'vendor/celeste-game/Content/Maps';
// Names the simulator's map decoder already turns into a real EntityKind.
const decoded = new Set([
  'jumpThru', 'dreamBlock', 'spikesUp', 'spikesDown', 'spikesLeft', 'spikesRight', 'water', 'booster',
  'redBooster', 'infiniteStar', 'flyFeather', 'bigSpinner', 'fireBall', 'puffer', 'oshiroBoss', 'seeker',
  'snowball', 'killbox', 'cloud', 'badelineBoost', 'spring', 'wallSpringLeft', 'wallSpringRight',
  'strawberry', 'refill', 'fallingBlock', 'exitBlock', 'invisibleBarrier', 'windTrigger', 'bounceBlock',
  'theoCrystal', 'blackGem', 'heartGem', 'risingLava', 'sandwichLava', 'glider', 'zipMover', 'moveBlock',
  'templeGate', 'cassetteBlock', 'spinner', 'towerviewer', 'lookout', 'crushBlock', 'dashBlock',
  'wallBooster', 'coreModeToggle', 'SummitBackgroundManager',
]);
const candidates = [
  // Solid subclasses the decoder does not handle
  'lockBlock', 'swapBlock', 'starJumpBlock', 'goldenBlock', 'floatySpaceBlock', 'glassBlock',
  'crumblePlatform', 'crumbleWallOnRumble', 'dashSwitch', 'switchGate', 'ridgeGate', 'mrOshiroDoor',
  'clutterDoor', 'clutterSwitch', 'clutterBlock', 'seekerBarrier', 'templeCrackedBlock',
  'theoCrystalPedestal', 'gondola', 'plateau', 'introCrusher', 'introPavement', 'fireBarrier',
  'bridgeFixed', 'lightningBreakerBox', 'resortRoofEnding', 'negaBlock', 'finalBossMovingBlock',
  // Trigger subclasses
  'stopBoostTrigger', 'dummyTrigger', 'triggerSpikes', 'altarTrigger', 'respawnTargetTrigger',
  'detachStrawberryTrigger', 'interactTrigger', 'musicTrigger', 'cameraTargetTrigger', 'altMusicTrigger',
  'moonGlitchBackgroundTrigger', 'blackHoleTrigger', 'glitchTrigger', 'summitGem',
];
const files = fs.readdirSync(mapsDir).filter((f) => f.endsWith('.bin'));
const hits = new Map();
for (const f of files) {
  const text = fs.readFileSync(path.join(mapsDir, f)).toString('latin1');
  for (const name of candidates) {
    if (decoded.has(name)) continue;
    if (text.includes(name)) {
      if (!hits.has(name)) hits.set(name, []);
      hits.get(name).push(f.replace('.bin', ''));
    }
  }
}
console.log(`maps scanned: ${files.length}`);
for (const [name, where] of [...hits.entries()].sort((a, b) => a[0].localeCompare(b[0]))) {
  console.log(`${name}\t${where.length} maps\t${where.slice(0, 6).join(', ')}${where.length > 6 ? ', ...' : ''}`);
}
console.log(`\ncandidates with no hit: ${candidates.filter((c) => !hits.has(c) && !decoded.has(c)).join(', ')}`);
