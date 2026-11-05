import { useEffect, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import type { FormEvent, ReactNode } from "react";
import { Activity, BarChart3, Bell, ChevronDown, CircleHelp, Clock3, LayoutDashboard, Plus, Settings2, ShieldCheck, SlidersHorizontal, Wallet, X, Zap } from "lucide-react";

type Page = "Overview" | "Strategies" | "History" | "Settings";
type Strategy = {id:string; base:string; quote:string; venues:string[]};
type NativeStrategy = {id:string; base_token:string; quote_token:string; venues:string[]};
type AuthState = {has_password:boolean; authenticated:boolean};
type WalletState = {connected:boolean; address:string|null};
const navigation: {label:Page; icon:typeof LayoutDashboard}[] = [
  {label:"Overview",icon:LayoutDashboard},{label:"Strategies",icon:SlidersHorizontal},{label:"History",icon:Clock3},{label:"Settings",icon:Settings2}
];

export default function App() {
  const desktop = isTauri();
  const [page,setPage]=useState<Page>("Overview");
  const [strategies,setStrategies]=useState<Strategy[]>([]);
  const [modal,setModal]=useState(false);
  const [monitoring,setMonitoring]=useState(false);
  const [toast,setToast]=useState("");
  const [authReady,setAuthReady]=useState(!desktop);
  const [auth,setAuth]=useState<AuthState>({has_password:false,authenticated:false});
  const [wallet,setWallet]=useState<WalletState>({connected:false,address:null});
  const [walletBusy,setWalletBusy]=useState(false);
  const notify=(s:string)=>{
    if(s.toLowerCase().includes("application is locked")){
      setAuth(old=>({...old,authenticated:false}));
      setStrategies([]);
      setWallet({connected:false,address:null});
    }
    setToast(s);window.setTimeout(()=>setToast(""),2600)
  };
  const refreshStrategies=async()=>{
    if(!desktop)return;
    const rows=await invoke<NativeStrategy[]>("list_strategies");
    setStrategies(rows.map(row=>({id:row.id,base:row.base_token,quote:row.quote_token,venues:row.venues})));
  };
  const refreshWallet=async()=>{
    if(!desktop)return;
    const status=await invoke<WalletState>("wallet_state");
    setWallet(status);
  };
  useEffect(()=>{
    if(!desktop)return;
    invoke<AuthState>("auth_state").then(async status=>{
      setAuth(status);
      if(status.authenticated) { await refreshStrategies(); await refreshWallet(); }
    }).catch(error=>notify(String(error))).finally(()=>setAuthReady(true));
  },[]);
  const save=async(s:Strategy)=>{
    try {
      if(desktop){
        const row=await invoke<NativeStrategy>("create_strategy",{baseToken:s.base,quoteToken:s.quote,venues:s.venues});
        s={id:row.id,base:row.base_token,quote:row.quote_token,venues:row.venues};
      }
      setStrategies(old=>[...old,s]);setModal(false);setPage("Strategies");notify(desktop?"Strategy saved locally":"Strategy added to your preview workspace");
    } catch(error){notify(String(error))}
  };
  const removeStrategy=async(id:string)=>{
    try { if(desktop) await invoke("delete_strategy",{id}); setStrategies(old=>old.filter(x=>x.id!==id)); }
    catch(error){notify(String(error))}
  };
  const lock=async()=>{
    try { await invoke("lock_app"); setAuth(old=>({...old,authenticated:false})); setStrategies([]); setWallet({connected:false,address:null}); }
    catch(error){notify(String(error))}
  };
  const authenticated= !desktop || auth.authenticated;
  const importWallet=async()=>{
    if(!desktop){notify("Wallet import is available in the Windows desktop app.");return}
    setWalletBusy(true);
    try { const status=await invoke<WalletState>("import_wallet"); setWallet(status); notify("Wallet imported into Windows Credential Manager."); }
    catch(error){ if(!String(error).toLowerCase().includes("cancelled")) notify(String(error)); }
    finally { setWalletBusy(false); }
  };
  const removeWallet=async()=>{
    if(!desktop || !window.confirm("Remove the imported wallet from this ArbX installation? This deletes its saved private key from Windows Credential Manager.")) return;
    setWalletBusy(true);
    try { await invoke("remove_wallet"); setWallet({connected:false,address:null}); notify("Wallet removed from this installation."); }
    catch(error){notify(String(error))}
    finally {setWalletBusy(false)}
  };
  useEffect(()=>{
    if(!desktop || !auth.authenticated) return;
    let timer=window.setTimeout(expire,15*60*1000);
    function expire(){
      void invoke("lock_app").then(()=>{
        setAuth(old=>({...old,authenticated:false}));
        setStrategies([]);
        setWallet({connected:false,address:null});
        setToast("Workspace locked after 15 minutes of inactivity.");
      }).catch(()=>{});
    }
    function reset(){window.clearTimeout(timer);timer=window.setTimeout(expire,15*60*1000)}
    const events: (keyof WindowEventMap)[]=["pointerdown","pointermove","keydown","wheel","touchstart"];
    events.forEach(event=>window.addEventListener(event,reset,{passive:true}));
    return ()=>{window.clearTimeout(timer);events.forEach(event=>window.removeEventListener(event,reset));};
  },[desktop,auth.authenticated]);
  if(desktop && !authReady) return <div className="auth-screen"><div className="auth-card"><div className="brand-mark">A</div><h1>Opening ArbX</h1><p>Preparing your local workspace…</p></div></div>;
  if(desktop && !authenticated) return <AuthGate hasPassword={auth.has_password} onSuccess={async()=>{const status=await invoke<AuthState>("auth_state");setAuth(status);await refreshStrategies();await refreshWallet();}}/>;
  return <div className="shell">
    <aside className="sidebar">
      <div className="brand"><div className="brand-mark">A</div><div><b>ARB<span>X</span></b><small>ARBITRAGE WORKSTATION</small></div></div>
      <div className="section-label">WORKSPACE</div>
      <nav>{navigation.map(({label,icon:Icon})=><button key={label} className={"nav-item "+(page===label?"selected":"")} onClick={()=>setPage(label)}><Icon size={17}/>{label}{label==="Strategies"&&strategies.length>0&&<i>{strategies.length}</i>}</button>)}</nav>
      <div className="sidebar-bottom"><div className="network-card"><small><i/> NETWORK</small><strong>Polygon PoS</strong><span>CHAIN ID 137</span><em>DEMO ENVIRONMENT</em></div><button className="help" onClick={()=>notify("Documentation will be added in a later milestone")}><CircleHelp size={15}/>Help & documentation</button><div className="version">ARBX DESKTOP <span>0.1.0 · PREVIEW</span></div></div>
    </aside>
    <main>
      <header><div className="crumb">ARBX <b>/</b> {page.toUpperCase()}</div><div className="top-actions"><span className="local-tag">● LOCAL PREVIEW</span><button className="icon-btn" onClick={()=>notify("You're all caught up")}><Bell size={17}/></button><i className="divider"/>{desktop?<button className="profile lock-profile" onClick={lock}><span>A</span> LOCK WORKSPACE <ShieldCheck size={13}/></button>:<div className="profile"><span>A</span> Local workspace <ChevronDown size={13}/></div>}</div></header>
      <div className="content">
        <div className="heading"><div><small>{page==="Overview"?"THURSDAY, NOVEMBER 06, 2026":"ARBX / "+page.toUpperCase()}</small><h1>{page==="Overview"?"Overview":page==="History"?"Trade history":page}</h1><p>{page==="Overview"?"Your arbitrage desk at a glance.":page==="Strategies"?"Define token pairs and venues to compare.":page==="History"?"Review execution records and realised outcomes.":"Manage local application preferences."}</p></div>{page==="Overview"?<button className={"bot-button "+(monitoring?"on":"")} onClick={()=>setMonitoring(v=>!v)}><i/> {monitoring?"MONITORING ON":"BOT OFF"} <ChevronDown size={13}/></button>:page==="Strategies"?<button className="primary" onClick={()=>setModal(true)}><Plus size={15}/> NEW STRATEGY</button>:null}</div>
        {page==="Overview"&&<><div className="notice"><Zap size={17}/><div><b>Preview environment</b><span>Demo interface only. No wallet is connected and no blockchain transactions can be made.</span></div><strong>NO LIVE TRADING</strong></div>
          <div className="metrics"><Metric label="PORTFOLIO VALUE" value="$12,840.50" sub="DEMO BALANCE" icon={<Wallet size={17}/>}/><Metric label="TOTAL P&L" value="+$840.50" sub="DEMO PERFORMANCE" icon={<BarChart3 size={17}/>}/><Metric label="STRATEGIES" value={String(strategies.length).padStart(2,"0")} sub="LOCAL CONFIGURATION" icon={<SlidersHorizontal size={17}/>}/><Metric label="OPPORTUNITIES" value="—" sub="NO LIVE SCANNER" icon={<Activity size={17}/>}/></div>
          <div className="columns"><section className="panel"><div className="panel-head"><div><small>PERFORMANCE</small><h2>Portfolio performance</h2></div><span className="select">7 DAYS⌄</span></div><div className="chart-value">$12,840.50 <em>↗ 2.41%</em></div><div className="chart"><div className="y-labels"><span>$13k</span><span>$12.5k</span><span>$12k</span><span>$11.5k</span></div><svg viewBox="0 0 680 180" preserveAspectRatio="none"><defs><linearGradient id="fill" x1="0" x2="0" y1="0" y2="1"><stop offset="0" stopColor="#e58a42" stopOpacity=".24"/><stop offset="1" stopColor="#e58a42" stopOpacity="0"/></linearGradient></defs>{[15,62,109,156].map(y=><line key={y} x1="0" y1={y} x2="680" y2={y} stroke="#292c2d" strokeDasharray="3 5"/>)}<path d="M0 145 C35 139 42 116 78 124 S130 135 160 109 S205 121 236 95 S281 108 315 84 S360 98 395 69 S435 83 470 61 S510 80 545 47 S590 57 625 35 S657 44 680 19 L680 180 L0 180Z" fill="url(#fill)"/><path d="M0 145 C35 139 42 116 78 124 S130 135 160 109 S205 121 236 95 S281 108 315 84 S360 98 395 69 S435 83 470 61 S510 80 545 47 S590 57 625 35 S657 44 680 19" fill="none" stroke="#e58a42" strokeWidth="2.5"/><circle cx="680" cy="19" r="4" fill="#f0a15a"/></svg></div><div className="chart-labels"><span>OCT 31</span><span>NOV 01</span><span>NOV 02</span><span>NOV 03</span><span>NOV 04</span><span>NOV 05</span><span>NOV 06</span></div><div className="chart-note">— ILLUSTRATIVE DATA · LIVE PORTFOLIO TRACKING NOT CONNECTED</div></section>
          <section className="panel activity"><div className="panel-head"><div><small>SYSTEM STATUS</small><h2>Activity</h2></div><span className="idle">● ALL SYSTEMS IDLE</span></div><div className="standby"><div><Activity size={22}/></div><b>Standing by</b><p>{monitoring?"Demo monitoring toggled on. No network requests are running.":"Your workspace is ready when you are."}</p></div><div className="activity-row"><ShieldCheck size={15}/><span><b>Safety mode enabled</b><small>Live execution unavailable</small></span><em>NOW</em></div><div className="activity-row"><Wallet size={15}/><span><b>Polygon workspace selected</b><small>Chain configuration preview</small></span><em>LOCAL</em></div></section></div>
          <section className="panel strategies-panel"><div className="panel-head"><div><small>YOUR CONFIGURATION</small><h2>Strategies <i>{strategies.length}</i></h2></div><button className="link-btn" onClick={()=>setPage("Strategies")}>VIEW ALL ↗</button></div>{strategies.length===0?<div className="empty"><div><Plus size={23}/></div><b>Nothing on the board yet</b><p>Create your first token-pair strategy to start shaping your arbitrage workspace.</p><button className="primary" onClick={()=>setModal(true)}><Plus size={15}/> CREATE STRATEGY</button></div>:strategies.map(s=><StrategyRow key={s.id} strategy={s}/>)}</section>
        </>}
        {page==="Strategies"&&<><div className="notice"><ShieldCheck size={17}/><div><b>Configuration preview</b><span>Strategies exist in temporary UI state only and are lost when the app reloads.</span></div><strong>DEMO</strong></div>{strategies.length===0?<section className="panel empty full"><div><Plus size={23}/></div><b>Empty</b><p>No strategies have been created. Add a token pair to configure your workspace.</p><button className="primary" onClick={()=>setModal(true)}><Plus size={15}/> ADD FIRST STRATEGY</button></section>:<section className="panel list">{strategies.map(s=><div className="strategy-row" key={s.id}><div><b>{s.base.slice(0,9)}… / {s.quote.slice(0,9)}…</b><small>Polygon PoS · {s.venues.join(" / ")}</small></div><span>DRAFT</span><button onClick={()=>removeStrategy(s.id)} aria-label="Delete strategy"><X size={15}/></button></div>)}</section>}</>}
        {page==="History"&&<><div className="notice"><Clock3 size={17}/><div><b>History is not connected</b><span>Trade records will appear after the trading engine and local database are implemented.</span></div><strong>NO TRADES</strong></div><section className="panel empty full"><div><Clock3 size={22}/></div><b>No trades recorded</b><p>Confirmed and reconciled trades will appear here. Rejected opportunities will never be labelled as completed trades.</p></section></>}
        {page==="Settings"&&<div className="settings-grid">
          <Setting icon={<ShieldCheck size={18}/>} title="Application security" text="Password-protected local workspace with automatic locking after 15 minutes of inactivity. Wallet secrets are kept outside the application database." status="LOCAL SECURITY ACTIVE"/>
          <section className="panel setting wallet-setting"><i><Wallet size={18}/></i><div className="wallet-info"><b>Wallet</b><p>{wallet.connected&&wallet.address?<>Imported wallet <strong>{wallet.address.slice(0,8)}…{wallet.address.slice(-6)}</strong>. The private key is held in Windows Credential Manager and is never returned to the interface.</>:"No wallet is currently imported. Import opens a native Windows secure credential prompt; the private key is not typed into ArbX's web interface."}</p><small>{wallet.connected?"WALLET CONNECTED · POLYGON EVM ADDRESS":"WALLET NOT CONNECTED"}</small><div className="wallet-actions"><button className="secondary" onClick={importWallet} disabled={!desktop||walletBusy}>{walletBusy?"PLEASE WAIT":wallet.connected?"REPLACE WALLET":"IMPORT WALLET"}</button>{wallet.connected&&<button className="text-button" onClick={removeWallet} disabled={walletBusy}>REMOVE</button>}</div></div></section>
          <Setting icon={<Activity size={18}/>} title="Network" text="Polygon PoS · Chain ID 137. Network reads and trading are not connected yet." status="INITIAL NETWORK"/>
          <Setting icon={<CircleHelp size={18}/>} title="About ArbX" text="A local workstation for evaluating same-chain token-pair arbitrage opportunities." status="VERSION 0.1.0 PREVIEW"/>
        </div>}
        <footer><span>ARBX / LOCAL DESKTOP PREVIEW</span><span>MARKET DATA NOT CONNECTED　●　EXECUTION DISABLED</span></footer>
      </div>
    </main>
    {modal&&<CreateStrategy close={()=>setModal(false)} save={save}/>}
    {toast&&<div className="toast"><ShieldCheck size={16}/>{toast}</div>}
  </div>;
}

function Metric({label,value,sub,icon}:{label:string;value:string;sub:string;icon:ReactNode}){return <div className="metric"><div><span>{label}</span>{icon}</div><strong>{value}</strong><small>{sub}</small></div>}
function StrategyRow({strategy}:{strategy:Strategy}){return <div className="strategy-row"><div><b>{strategy.base.slice(0,9)}… / {strategy.quote.slice(0,9)}…</b><small>{strategy.venues.join(" / ")}</small></div><span>POLYGON POS</span><em>DRAFT</em></div>}
function Setting({icon,title,text,status}:{icon:ReactNode;title:string;text:string;status:string}){return <section className="panel setting"><i>{icon}</i><div><b>{title}</b><p>{text}</p><small>{status}</small></div></section>}
function CreateStrategy({close,save}:{close:()=>void;save:(s:Strategy)=>void}){
  const [base,setBase]=useState("");const [quote,setQuote]=useState("");const [venues,setVenues]=useState(["Demo Venue A","Demo Venue B"]);const [error,setError]=useState("");
  const toggle=(v:string)=>setVenues(old=>old.includes(v)?old.filter(x=>x!==v):[...old,v]);
  const submit=(e:FormEvent)=>{e.preventDefault();if(!/^0x[a-fA-F0-9]{40}$/.test(base)||!/^0x[a-fA-F0-9]{40}$/.test(quote)){setError("Enter two valid 20-byte EVM token addresses.");return}if(base.toLowerCase()===quote.toLowerCase()){setError("Token addresses must be different.");return}if(venues.length<2){setError("Choose at least two venues.");return}save({id:crypto.randomUUID(),base,quote,venues})};
  return <div className="backdrop" onMouseDown={e=>e.target===e.currentTarget&&close()}><div className="modal"><div className="modal-head"><div><small>NEW CONFIGURATION</small><h2>Create strategy</h2><p>Configure a pair for same-chain venue comparison.</p></div><button onClick={close}><X size={18}/></button></div><form onSubmit={submit}><label>NETWORK <span>FIXED FOR V1</span></label><div className="field">Polygon PoS <small>CHAIN ID 137</small></div><label htmlFor="base">BASE TOKEN ADDRESS</label><input id="base" value={base} onChange={e=>setBase(e.target.value.trim())} placeholder="0x…" autoComplete="off" spellCheck={false}/><label htmlFor="quote">QUOTE TOKEN ADDRESS</label><input id="quote" value={quote} onChange={e=>setQuote(e.target.value.trim())} placeholder="0x…" autoComplete="off" spellCheck={false}/><label>COMPARISON VENUES <span>SELECT AT LEAST TWO</span></label><div className="venues">{["Demo Venue A","Demo Venue B","Demo Venue C"].map(v=><button type="button" key={v} className={venues.includes(v)?"chosen":""} onClick={()=>toggle(v)}>{venues.includes(v)?"✓":"+"} {v}<small>MOCK</small></button>)}</div><p className="disclaimer">Demo only. No addresses are queried and no transactions can be executed.</p>{error&&<p className="error">{error}</p>}<div className="modal-actions"><button type="button" className="secondary" onClick={close}>CANCEL</button><button type="submit" className="primary">SAVE STRATEGY ↗</button></div></form></div></div>
}


function AuthGate({hasPassword,onSuccess}:{hasPassword:boolean;onSuccess:()=>Promise<void>}){
  const [password,setPassword]=useState("");
  const [confirm,setConfirm]=useState("");
  const [error,setError]=useState("");
  const [busy,setBusy]=useState(false);
  const submit=async(event:FormEvent)=>{
    event.preventDefault();setError("");
    if(!hasPassword && password!==confirm){setError("The passwords do not match.");return}
    setBusy(true);
    try {
      await invoke(hasPassword?"login":"setup_password",{password});
      await onSuccess();
    } catch(reason){setError(String(reason))}
    finally{setBusy(false)}
  };
  return <div className="auth-screen"><form className="auth-card" onSubmit={submit}>
    <div className="brand-mark">A</div><small>ARBX / LOCAL SECURITY</small>
    <h1>{hasPassword?"Welcome back":"Secure your workspace"}</h1>
    <p>{hasPassword?"Unlock ArbX to access your locally stored strategies.":"Create the password required to unlock this ArbX installation."}</p>
    <label htmlFor="app-password">{hasPassword?"APP PASSWORD":"CREATE PASSWORD"}</label>
    <input id="app-password" type="password" value={password} onChange={e=>setPassword(e.target.value)} autoComplete={hasPassword?"current-password":"new-password"} minLength={12} required autoFocus/>
    {!hasPassword&&<><label htmlFor="confirm-password">CONFIRM PASSWORD</label><input id="confirm-password" type="password" value={confirm} onChange={e=>setConfirm(e.target.value)} autoComplete="new-password" minLength={12} required/><span className="auth-hint">Use at least 12 characters. Your password is stored as a salted Argon2id hash.</span></>}
    {error&&<div className="auth-error">{error}</div>}
    <button className="primary auth-submit" disabled={busy}>{busy?"PLEASE WAIT":hasPassword?"UNLOCK WORKSPACE":"CREATE PASSWORD"}</button>
    <span className="auth-foot">LOCAL DEVICE AUTHENTICATION · NO ACCOUNT REQUIRED</span>
  </form></div>
}
