// fable r0 lens 2 harness: feed SH2-composed descriptors through the SAME
// libnunchuk entry points Nunchuk Desktop 2.1.1 calls when a user imports a
// wallet from a descriptor (qUtils::ParseWalletDescriptor ->
// nunchuk::Utils::ParseWalletDescriptor -> ParseDescriptors), then re-render
// the wallet the way the app stores it and derive addresses through the
// embedded Core exactly as CoreUtils does.
//
// stdin: one "<name>\t<descriptor>" per line. stdout: a block per line.
#include <nunchuk.h>
#include <descriptor.h>
#include <coreutils.h>
#include <miniscript/util.h>
#include <embeddedrpc.h>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

using namespace nunchuk;

static const char* wt(WalletType t) {
  switch (t) {
    case WalletType::SINGLE_SIG: return "SINGLE_SIG";
    case WalletType::MULTI_SIG: return "MULTI_SIG";
    case WalletType::ESCROW: return "ESCROW";
    case WalletType::MINISCRIPT: return "MINISCRIPT";
  }
  return "?";
}
static const char* at(AddressType a) {
  switch (a) {
    case AddressType::ANY: return "ANY";
    case AddressType::LEGACY: return "LEGACY";
    case AddressType::NESTED_SEGWIT: return "NESTED_SEGWIT";
    case AddressType::NATIVE_SEGWIT: return "NATIVE_SEGWIT";
    case AddressType::TAPROOT: return "TAPROOT";
  }
  return "?";
}
static const char* tt(WalletTemplate t) {
  switch (t) {
    case WalletTemplate::DEFAULT: return "DEFAULT";
    case WalletTemplate::DISABLE_KEY_PATH: return "DISABLE_KEY_PATH";
  }
  return "?";
}

static std::string tree(const ScriptNode& n, int depth = 0) {
  std::ostringstream ss;
  ss << std::string(depth * 2, ' ') << "type=" << (int)n.get_type() << " k=" << n.get_k()
     << " keys=" << n.get_keys().size() << " id=";
  for (auto i : n.get_id()) ss << i << ".";
  ss << "\n";
  for (auto& s : n.get_subs()) ss << tree(s, depth + 1);
  return ss.str();
}

static void dump(const Wallet& w) {
  std::cout << "  wallet_type=" << wt(w.get_wallet_type())
            << " address_type=" << at(w.get_address_type())
            << " template=" << tt(w.get_wallet_template())
            << " m=" << w.get_m() << " n=" << w.get_n()
            << " id=" << w.get_id() << "\n";
  int i = 0;
  for (auto& s : w.get_signers()) {
    auto eii = s.get_external_internal_index();
    std::cout << "  signer[" << i++ << "] xfp=" << s.get_master_fingerprint()
              << " path=" << s.get_derivation_path() << " eii=" << eii.first << ";" << eii.second
              << " xpub=" << s.get_xpub() << "\n";
  }
  std::cout << "  desc.EXTERNAL_ALL=" << w.get_descriptor(DescriptorPath::EXTERNAL_ALL) << "\n";
  std::cout << "  desc.INTERNAL_ALL=" << w.get_descriptor(DescriptorPath::INTERNAL_ALL) << "\n";
  std::cout << "  desc.EXTERNAL_INTERNAL=" << w.get_descriptor(DescriptorPath::EXTERNAL_INTERNAL) << "\n";
  std::cout << "  desc.TEMPLATE=" << w.get_descriptor(DescriptorPath::TEMPLATE) << "\n";
  if (w.get_wallet_type() == WalletType::MINISCRIPT) {
    std::string ms = w.get_miniscript();
    std::cout << "  miniscript=" << ms << "\n";
    try {
      std::vector<std::string> keypath;
      auto node = Utils::GetScriptNode(ms, keypath);
      std::cout << "  scriptnode=" << ScriptNodeToString(node) << "\n";
      std::cout << tree(node);
      auto paths = Utils::GetAllSigningPaths(ms);
      std::cout << "  signing_paths=" << paths.size() << ":";
      for (auto& p : paths) {
        std::cout << " [";
        for (auto& id : p) { for (auto i : id) std::cout << i << "."; std::cout << ","; }
        std::cout << "]";
      }
      std::cout << "\n";
    } catch (std::exception& e) {
      std::cout << "  scriptnode: EXC " << e.what() << "\n";
    }
  }
  try {
    w.check_valid();
    std::cout << "  check_valid=ok\n";
  } catch (BaseException& e) {
    std::cout << "  check_valid=REFUSE code=" << e.code() << " what='" << e.what() << "'\n";
  }
  try {
    auto san = Utils::SanitizeSingleSigners(w.get_signers());
    std::cout << "  sanitize_signers=" << san.size() << "\n";
  } catch (BaseException& e) {
    std::cout << "  sanitize_signers=REFUSE code=" << e.code() << " what='" << e.what() << "'\n";
  }
  for (auto path : {DescriptorPath::EXTERNAL_ALL, DescriptorPath::INTERNAL_ALL}) {
    try {
      auto addrs = CoreUtils::getInstance().DeriveAddresses(w.get_descriptor(path), 0, 2);
      std::cout << "  " << (path == DescriptorPath::EXTERNAL_ALL ? "receive" : "change") << "=";
      for (auto& a : addrs) std::cout << a << " ";
      std::cout << "\n";
    } catch (std::exception& e) {
      std::cout << "  derive: EXC " << e.what() << "\n";
    }
  }
}

int main() {
  EmbeddedRpc::getInstance().Init("main");
  Utils::SetChain(Chain::MAIN);
  CoreUtils::getInstance().SetChain(Chain::MAIN);
  std::string line;
  while (std::getline(std::cin, line)) {
    if (line.empty()) continue;
    auto tab = line.find('\t');
    std::string name = line.substr(0, tab), desc = line.substr(tab + 1);
    std::cout << "### " << name << "\n";
    // Stage 1: the low-level parser, with its error string.
    {
      std::string err;
      auto w = ParseDescriptors(desc, err);
      std::cout << "  ParseDescriptors=" << (w ? "ACCEPT" : "REFUSE") << " error='" << err << "'\n";
    }
    // Stage 1b: the template validator directly, so a swallowed exception is visible.
    if (desc.rfind("wsh(", 0) == 0 || desc.rfind("tr(", 0) == 0) {
      std::string inner = desc.substr(0, desc.find('#'));
      bool is_tr = inner.rfind("tr(", 0) == 0;
      try {
        if (is_tr) {
          std::string e2;
          bool ok = Utils::IsValidTapscriptTemplate(inner, e2);
          std::cout << "  IsValidTapscriptTemplate=" << (ok ? "true" : "false") << " error='" << e2 << "'\n";
        } else {
          inner = inner.substr(4, inner.size() - 5);
          bool ok = Utils::IsValidMiniscriptTemplate(inner, AddressType::NATIVE_SEGWIT);
          std::cout << "  IsValidMiniscriptTemplate=" << (ok ? "true" : "false") << "\n";
        }
      } catch (BaseException& e) {
        std::cout << "  IsValid*Template=EXC code=" << e.code() << " what='" << e.what() << "'\n";
      } catch (std::exception& e) {
        std::cout << "  IsValid*Template=EXC std what='" << e.what() << "'\n";
      }
    }
    // Stage 2: what the app calls.
    try {
      Wallet w = Utils::ParseWalletDescriptor(desc);
      std::cout << "  ParseWalletDescriptor=ACCEPT\n";
      dump(w);
    } catch (BaseException& e) {
      std::cout << "  ParseWalletDescriptor=REFUSE code=" << e.code() << " what='" << e.what() << "'\n";
    } catch (std::exception& e) {
      std::cout << "  ParseWalletDescriptor=REFUSE std what='" << e.what() << "'\n";
    }
  }
  return 0;
}
