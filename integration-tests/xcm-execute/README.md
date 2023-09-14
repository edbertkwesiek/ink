# `xcm-execute` example

## What is this example about?

It demonstrates how to use XCM from an ink! contract.

## Chain-side configuration

To integrate this example into Substrate you need to implement `pallet-xcm` and configure the `Xcm` trait of `pallet-contracts`

  ```rust
// In your node's runtime configuration file (runtime.rs)
impl pallet_xcm::Config for Runtime {
    // ...
}

impl pallet_contracts::Config for Runtime {
    // …
    type Xcm = PalletXCMAdapter<Self>;
    // …
 }
  ```
