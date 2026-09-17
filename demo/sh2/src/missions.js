// The mission definitions.
//
// EVERY STEP AND EVERY TAP COUNT HERE WAS WALKED against the real firmware wasm
// and the screen read back after each one; see WALKS.md for the transcript and
// the traps. Nothing in this file is inferred from source, and a step nobody
// has driven does not belong here -- a wrong step strands a stranger with no
// way to tell whether they or the device got it wrong.
//
// Device buttons, by position (the page maps taps to device coordinates):
//   top-right     = back          middle-left  = previous entry
//   middle-right  = next / page   bottom-right = confirm
export const BUTTONS = {
  back: "top-right", prev: "middle-left", next: "middle-right", ok: "bottom-right",
};

export const MISSIONS = [
  {
    id: "decaying",
    title: "Build a decaying multisig",
    blurb: "Three spend paths that unlock in stages: 2-of-2 now, a wider set after 26,280 blocks, a single key after a fixed block height.",
    taps: 19, wrapper: "Taproot", measured: true,
    why: "This is the shape people actually want and rarely build: a wallet that degrades gracefully instead of failing closed. Nothing here is typed — the preset arrives fully formed.",
    steps: [
      'At <b>"A systemwide payload is present. Load it?"</b> tap <b>SKIP</b>, then <kbd>confirm</kbd>.',
      'Tap <kbd>previous</kbd> (middle-left) <b>5 times</b> to reach <b>Wallet Policy</b>. It wraps backwards, so this is shorter than going forward.',
      'Tap <kbd>confirm</kbd> to enter.',
      'Choose <b>Build a new policy</b>, then <kbd>confirm</kbd>.',
      'Choose <b>Taproot (tr)</b>, then <kbd>confirm</kbd>.',
      'Choose <b>decaying-multisig</b>, then <kbd>confirm</kbd>.',
      'The policy is already built: <b>slots: 4</b>, three spend paths. Tap <kbd>next</kbd> once — <b>"Done" is on page 2</b> and page 1 gives no hint it is there.',
      'Choose <b>Done</b>, then <kbd>confirm</kbd>.',
    ],
    ends: "A <b>Template-ID</b> — a 128-bit fingerprint of the policy shape — plus the exact <code>mk encode</code> command each cosigner runs to produce their key card.",
  },
  {
    id: "hashlock",
    title: "Build a key + hashlock-phrase wallet",
    blurb: "One path needs a key AND a secret phrase. The other needs a different key, after a wait.",
    taps: 20, wrapper: "Segwit", measured: true,
    why: "A hashlock is a second factor that is not a key: something you remember, committed to on-chain. Useful for a duress path, or for handing someone access without handing them a key.",
    steps: [
      'At <b>"A systemwide payload is present. Load it?"</b> tap <b>SKIP</b>, then <kbd>confirm</kbd>.',
      'Tap <kbd>previous</kbd> <b>5 times</b> to reach <b>Wallet Policy</b>, then <kbd>confirm</kbd>.',
      'Choose <b>Build a new policy</b>, then <kbd>confirm</kbd>.',
      'Choose <b>Segwit (wsh)</b>, then <kbd>confirm</kbd>.',
      'Choose <b>hashlock-gated</b>, then <kbd>confirm</kbd>.',
      'Read the two paths: <b>Path 1: 1 key + hash</b> and <b>Path 2: 1 key + 26280 blocks</b>. With only two paths, <b>Done is visible on page 1</b>.',
      'Choose <b>Done</b>, then <kbd>confirm</kbd>.',
    ],
    ends: "A Template-ID for a wallet whose first path cannot be spent by a key alone.",
  },
  {
    id: "zenhodl",
    title: "Build a zen-hodl wallet",
    blurb: "One key, and it cannot spend anything younger than 32,768 blocks — about 227 days.",
    taps: 42, wrapper: "Taproot", measured: true, long: true,
    why: "Not a multisig at all: a single key you deliberately cannot use in a hurry. You build this one path-by-path rather than from a preset, so it is the longest of the three — and the one that shows the composer is not just a menu of canned shapes.",
    steps: [
      'At <b>"A systemwide payload is present. Load it?"</b> tap <b>SKIP</b>, then <kbd>confirm</kbd>.',
      'Tap <kbd>previous</kbd> <b>5 times</b> to reach <b>Wallet Policy</b>, then <kbd>confirm</kbd>.',
      'Choose <b>Build a new policy</b> → <b>Taproot (tr)</b> → <b>Build my own paths</b> (the first row). You get an empty policy, <b>slots: 0</b>.',
      'Choose <b>Add a spend path</b>. At <b>"What can spend on this path?"</b> choose <b>Keys</b>.',
      'At <b>"how many keys?"</b> choose <b>1</b>. At <b>"how many must sign?"</b> choose <b>1</b>.',
      'Back on the list, open <b>Path 1</b>, then choose <b>Timelock</b>.',
      'Choose <b>After a wait</b>, then <b>Blocks</b>.',
      'Type <b>32768</b> on the keypad and <kbd>confirm</kbd>. The device answers <b>"32768 blocks (about 227.6 days)"</b>. The field caps at 65535.',
      '<kbd>confirm</kbd> once more — Path 1 now reads <b>1 key + 32768 blocks</b>.',
      'Tap <kbd>back</kbd> to return to the list. <b>Your path is kept</b> — back is the safe way out here.',
      'Choose <b>Done</b>, then <kbd>confirm</kbd>.',
    ],
    ends: "Template-ID <code>73c33a5dea17b45376a6246995df0cbb</code> — and you can check that number yourself, below.",
  },
  {
    id: "sealed",
    title: "Open a sealed backup",
    blurb: "A passphrase-encrypted payload holding a whole wallet: codex32 seed shares, cosigner key cards and wallet-policy cards.",
    taps: 60, wrapper: null, measured: true, advanced: true, long: true,
    why: "This is what a complete backup looks like on this device — secret and public material in one sealed container, opened by something you remember.",
    steps: [
      'At <b>"A systemwide payload is present. Load it?"</b> tap <b>SKIP</b>, then <kbd>confirm</kbd>.',
      'Tap <kbd>previous</kbd> <b>once</b>. <b>Sealed Payload</b> is the last entry, so going backwards reaches it immediately.',
      'Tap <kbd>confirm</kbd>. Read the screen: these 12 words are the payload\'s <b>passphrase</b>, not a seed — no wallet is derived from them.',
      'Tap <kbd>confirm</kbd> again for the keyboard, then type the passphrase below.',
      'You only need <b>3 or 4 letters per word</b>. The device shows a live match count and completes the word as soon as one candidate is left — "mos" is already MOSQUITO.',
      'Tap <kbd>confirm</kbd> after each completed word to move to the next.',
      'After the twelfth word it opens, straight onto the first plate: <b>SECRET seed material — ms1 1/3</b>.',
      'Step through with <b>Skip</b> to see what a whole backup looks like.',
    ],
    ends: "The container's contents as engravable plates — three codex32 seed shares first, then cosigner key cards. The seed plates are flagged <b>SECRET seed material</b>; the rest are public.",
  },
];
