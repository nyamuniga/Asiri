import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import logo from "./assets/logo.png";

type ShareDto = {
  index: number;
  data: string;
};

export default function App() {
  const [activeTab, setActiveTab] = useState<"split" | "recover">("split");

  return (
    <div className="min-h-screen flex flex-col items-center justify-center p-8">
      {/* Title / Logo Area */}
      <div className="mb-12 text-center flex flex-col items-center">
        <img src={logo} alt="Asiri Logo" className="w-24 h-24 mb-4 drop-shadow-[0_0_15px_rgba(102,252,241,0.5)] rounded-full border-2 border-asiri-neon/50 p-1" />
        <h1 className="text-6xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-asiri-neon to-purple-500 tracking-widest drop-shadow-[0_0_15px_rgba(102,252,241,0.5)]">
          ASIRI
        </h1>
        <p className="text-asiri-primary mt-2 uppercase tracking-[0.3em] text-sm font-bold">
          Advanced Secret Sharing
        </p>
      </div>

      {/* Tabs */}
      <div className="flex bg-asiri-surface/80 p-2 rounded-xl mb-8 border border-asiri-primary/30 shadow-[0_0_20px_rgba(31,40,51,0.8)] backdrop-blur-sm w-full max-w-2xl">
        <button
          className={`flex-1 py-3 text-lg font-bold rounded-lg transition-all ${
            activeTab === "split"
              ? "bg-asiri-primary/20 text-asiri-neon shadow-[0_0_10px_rgba(102,252,241,0.2)] border border-asiri-neon/50"
              : "text-gray-400 hover:text-asiri-neon hover:bg-white/5 border border-transparent"
          }`}
          onClick={() => setActiveTab("split")}
        >
          SPLIT
        </button>
        <button
          className={`flex-1 py-3 text-lg font-bold rounded-lg transition-all ${
            activeTab === "recover"
              ? "bg-asiri-primary/20 text-asiri-neon shadow-[0_0_10px_rgba(102,252,241,0.2)] border border-asiri-neon/50"
              : "text-gray-400 hover:text-asiri-neon hover:bg-white/5 border border-transparent"
          }`}
          onClick={() => setActiveTab("recover")}
        >
          RECOVER
        </button>
      </div>

      {/* Content Area */}
      <div className="w-full max-w-2xl bg-asiri-surface/90 backdrop-blur-md border border-asiri-primary/30 p-8 rounded-2xl shadow-2xl relative overflow-hidden">
        {/* Subtle geometric overlay for the content box */}
        <div className="absolute top-0 left-0 w-full h-full opacity-10 pointer-events-none bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI0MCIgaGVpZ2h0PSI0MCI+PHBhdGggZD0iTTIwIDBMNDAgMjBMMjAgNDBMMCAyMHoiIGZpbGw9Im5vbmUiIHN0cm9rZT0iIzY2RkNGMSIgc3Ryb2tlLXdpZHRoPSIyIi8+PC9zdmc+')] bg-repeat" />
        
        <div className="relative z-10">
          {activeTab === "split" ? <SplitView /> : <RecoverView />}
        </div>
      </div>
    </div>
  );
}

function SplitView() {
  const [secret, setSecret] = useState("");
  const [threshold, setThreshold] = useState(3);
  const [shares, setShares] = useState(5);
  const [result, setResult] = useState<ShareDto[] | null>(null);
  const [error, setError] = useState("");

  const handleSplit = async () => {
    setError("");
    setResult(null);
    try {
      const res = await invoke<ShareDto[]>("split_secret_cmd", {
        secret,
        threshold,
        shares,
      });
      setResult(res);
    } catch (e: any) {
      setError(e.toString());
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <label className="block text-asiri-primary text-sm font-bold mb-2">SECRET TO SPLIT</label>
        <textarea
          className="w-full bg-black/50 border border-asiri-primary/30 rounded p-4 text-white focus:outline-none focus:border-asiri-neon focus:ring-1 focus:ring-asiri-neon transition-all resize-none"
          rows={3}
          value={secret}
          onChange={(e) => setSecret(e.target.value)}
          placeholder="Enter a recovery phrase, private key, or password..."
        />
      </div>

      <div className="flex gap-4">
        <div className="flex-1">
          <label className="block text-asiri-primary text-sm font-bold mb-2">THRESHOLD</label>
          <input
            type="number"
            min={2}
            max={255}
            className="w-full bg-black/50 border border-asiri-primary/30 rounded p-4 text-white focus:outline-none focus:border-asiri-neon"
            value={threshold}
            onChange={(e) => setThreshold(parseInt(e.target.value))}
          />
        </div>
        <div className="flex-1">
          <label className="block text-asiri-primary text-sm font-bold mb-2">TOTAL SHARES</label>
          <input
            type="number"
            min={2}
            max={255}
            className="w-full bg-black/50 border border-asiri-primary/30 rounded p-4 text-white focus:outline-none focus:border-asiri-neon"
            value={shares}
            onChange={(e) => setShares(parseInt(e.target.value))}
          />
        </div>
      </div>

      <button
        onClick={handleSplit}
        className="w-full py-4 bg-gradient-to-r from-asiri-primary to-purple-600 text-white font-bold tracking-wider rounded border border-transparent hover:border-asiri-neon shadow-[0_0_15px_rgba(69,162,158,0.4)] hover:shadow-[0_0_25px_rgba(102,252,241,0.6)] transition-all uppercase"
      >
        Generate Shares
      </button>

      {error && <div className="text-red-400 bg-red-900/20 border border-red-500/50 p-4 rounded">{error}</div>}

      {result && (
        <div className="mt-8 space-y-4 animate-fade-in">
          <div className="bg-orange-900/20 border border-orange-500/50 p-4 rounded mb-6 shadow-[0_0_15px_rgba(234,88,12,0.1)]">
            <h4 className="text-orange-400 font-bold mb-2 flex items-center gap-2">
              ⚠️ CRITICAL SECURITY WARNING
            </h4>
            <p className="text-orange-300/80 text-sm leading-relaxed">
              For maximum security, <strong>do NOT save these shares digitally</strong> (e.g. taking a screenshot, saving in a text file, or emailing them to yourself). Write them down physically on a piece of paper or stamp them in metal, and store each share in a separate, secure geographic location.
            </p>
          </div>
          
          <h3 className="text-asiri-neon font-bold border-b border-asiri-primary/30 pb-2">GENERATED SHARES</h3>
          {result.map((s, i) => (
            <div key={i} className="bg-black/40 border border-asiri-primary/20 p-4 rounded flex gap-4 items-center">
              <span className="text-purple-400 font-bold bg-purple-900/30 px-3 py-1 rounded">#{s.index}</span>
              <span className="break-all text-sm text-gray-300 font-mono flex-1 select-all">{s.index}-{s.data}</span>
              <CopyButton text={`${s.index}-${s.data}`} />
            </div>
          ))}
          <p className="text-xs text-gray-500 italic mt-2">
            * For your security, the clipboard is automatically cleared 30 seconds after copying a share.
          </p>
        </div>
      )}
    </div>
  );
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
      
      // Auto-clear clipboard after 30 seconds
      setTimeout(async () => {
        try {
          const current = await navigator.clipboard.readText();
          if (current === text) {
            await navigator.clipboard.writeText("");
          }
        } catch (e) {
          // Fallback to blind clear if readText is blocked
          navigator.clipboard.writeText("");
        }
      }, 30000);
    } catch (err) {
      console.error("Failed to copy", err);
    }
  };

  return (
    <button
      onClick={handleCopy}
      className="ml-auto text-xs bg-asiri-primary/20 hover:bg-asiri-primary/40 text-asiri-neon px-3 py-2 rounded transition-colors border border-asiri-primary/30 font-bold tracking-wider shrink-0"
    >
      {copied ? "COPIED!" : "COPY"}
    </button>
  );
}

function RecoverView() {
  const [inputs, setInputs] = useState<string[]>([""]);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState("");

  const updateInput = (index: number, val: string) => {
    const newInputs = [...inputs];
    newInputs[index] = val;
    // Auto-add new empty row if typing in the last one
    if (index === inputs.length - 1 && val.trim() !== "") {
      newInputs.push("");
    }
    setInputs(newInputs);
  };

  const handleRecover = async () => {
    setError("");
    setResult(null);
    
    const parsedShares: ShareDto[] = [];
    for (const str of inputs) {
      const trimmed = str.trim();
      if (!trimmed) continue;
      
      const parts = trimmed.split("-");
      if (parts.length !== 2) {
        setError("Invalid format. Use <index>-<hex_data>");
        return;
      }
      parsedShares.push({
        index: parseInt(parts[0]),
        data: parts[1],
      });
    }

    if (parsedShares.length === 0) {
      setError("Please enter at least two shares.");
      return;
    }

    try {
      const res = await invoke<string>("recover_secret_cmd", { shares: parsedShares });
      setResult(res);
    } catch (e: any) {
      setError(e.toString());
    }
  };

  return (
    <div className="space-y-6">
      <label className="block text-asiri-primary text-sm font-bold mb-2">INPUT SHARES (Format: index-hex)</label>
      <div className="space-y-3 max-h-64 overflow-y-auto pr-2 custom-scrollbar">
        {inputs.map((val, i) => (
          <input
            key={i}
            type="text"
            className="w-full bg-black/50 border border-asiri-primary/30 rounded p-4 text-white focus:outline-none focus:border-asiri-neon font-mono text-sm"
            value={val}
            onChange={(e) => updateInput(i, e.target.value)}
            placeholder="e.g. 1-a7f4..."
          />
        ))}
      </div>

      <button
        onClick={handleRecover}
        className="w-full py-4 bg-gradient-to-r from-purple-600 to-asiri-primary text-white font-bold tracking-wider rounded border border-transparent hover:border-asiri-neon shadow-[0_0_15px_rgba(69,162,158,0.4)] hover:shadow-[0_0_25px_rgba(102,252,241,0.6)] transition-all uppercase"
      >
        Recover Secret
      </button>

      {error && <div className="text-red-400 bg-red-900/20 border border-red-500/50 p-4 rounded">{error}</div>}

      {result && (
        <div className="mt-8 space-y-4 animate-fade-in">
          <h3 className="text-asiri-neon font-bold border-b border-asiri-primary/30 pb-2">RECOVERED SECRET</h3>
          <div className="bg-black/60 border border-asiri-neon p-6 rounded text-center">
            <span className="text-white font-mono text-xl select-all break-all">{result}</span>
          </div>
        </div>
      )}
    </div>
  );
}
