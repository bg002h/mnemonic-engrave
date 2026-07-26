// Command rehearsal-blinky is the throwaway payload for phase 4 of
// scripts/pico2-bootkey-rehearsal.sh.
//
// Its only job is to give an unambiguous, eyes-on pass signal that a
// self-signed image was ACCEPTED by the RP2350 bootrom on a secure-boot-sealed
// board: if the LED blinks, the signature verified and your code is running.
// A rejected image never reaches main().
//
// Target the plain Pico 2 (-target pico2), NOT the Pico 2 W: on the W the LED
// hangs off the CYW43 wireless chip rather than a GPIO, so machine.LED does
// nothing there. Use the plain Pico 2 as the primary rehearsal board.
//
// This deliberately does nothing else -- no peripherals, no storage, no OTP
// access. It is a signature-acceptance canary, not a test of the SeedHammer
// firmware (which cannot run meaningfully on a Pico anyway).
package main

import (
	"machine"
	"time"
)

func main() {
	led := machine.LED
	led.Configure(machine.PinConfig{Mode: machine.PinOutput})

	// Distinctive 2-short-1-long pattern, so it cannot be confused with a
	// bootloader heartbeat or a stock demo image left on the board.
	for {
		for range 2 {
			led.High()
			time.Sleep(120 * time.Millisecond)
			led.Low()
			time.Sleep(180 * time.Millisecond)
		}
		led.High()
		time.Sleep(700 * time.Millisecond)
		led.Low()
		time.Sleep(600 * time.Millisecond)
	}
}
