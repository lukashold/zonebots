// Particle constructor and full native4028a0 traces, graphics calls intercepted.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.nio.file.Files;
import java.nio.file.Path;
public class ZNativeWorldEffectOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,O=0x1100000L,TRIG=O+0x1000;
 int seed,calls,dx,dy,sprite,height,angle,transformed;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void run(EmulatorHelper e,long entry,long ecx,long edx,long...args)throws Exception{
  dx=-999;dy=-999;sprite=-1;height=0;angle=0;transformed=0;e.writeRegister("ESP",S);put(e,S,4,RET);for(int i=0;i<args.length;i++)put(e,S+4+i*4,4,args[i]);e.writeRegister("ECX",ecx);e.writeRegister("EDX",edx);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),stack=e.readRegister("ESP").longValue();
   if(pc==0x460910)e.writeRegister("EAX",(seed*7919^(calls++)*2027^0x7731a5c9)&0xffffffffL);
   else if(pc==0x4666c0||pc==0x466ac0){dx=(short)e.readRegister("ECX").intValue();dy=(short)e.readRegister("EDX").intValue();sprite=(short)read(e,stack+4,4);if(pc==0x466ac0){height=(short)read(e,stack+8,4);angle=(int)read(e,stack+12,4);transformed=1;}}
   else if(pc!=0x40bb50)throw new Exception("Unexpected "+Long.toHexString(pc));long ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
  }
 }
 long hash(byte[]bytes){int h=0x811c9dc5;for(byte b:bytes)h=(h^(b&255))*16777619;return h&0xffffffffL;}
 void initialize(EmulatorHelper e,int kind,int seed)throws Exception{
  this.seed=seed;calls=0;e.writeMemory(toAddr(O),new byte[44]);put(e,O+6,2,seed<16?346:seed<24?-180:4080);put(e,O+8,2,seed<16?420:seed<24?-100:4080);put(e,O+22,1,kind==0?4:12);put(e,O+24,1,kind==0?0:5);put(e,O+26,2,kind==0?250:557);put(e,0x586a7c,2,kind==0?250:557);run(e,kind==0?0x402770:0x4017b0,0,0,O);
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{0x460910,0x4666c0,0x466ac0,0x40bb50})e.setBreakpoint(toAddr(a));byte[]raw=Files.readAllBytes(Path.of("C:/GOG Games/Z/SINCOS.DAT"));for(int i=0;i<640;i++){int v=(short)(((raw[i*2]&255)<<8)|(raw[i*2+1]&255));put(e,TRIG+i*2,2,v>>5);}put(e,0x586a80,4,TRIG);
  put(e,0x477054,2,471);put(e,0x47705c,2,-256);put(e,0x477058,2,425);put(e,0x477060,2,1);put(e,0x622cb8,4,0);put(e,0x622d50,2,4096);put(e,0x622d52,2,4096);
  for(int kind=0;kind<2;kind++)for(seed=0;seed<64;seed++){initialize(e,kind,seed);out.println("{\"kind\":\"effectInit\",\"scenario\":"+(kind*64+seed)+",\"calls\":"+calls+",\"hash\":"+hash(e.readMemory(toAddr(O),44))+"}");}
  int s=0;for(int kind=0;kind<2;kind++)for(seed=0;seed<32;seed++){initialize(e,kind,seed);int[]clip=(seed>>2&3)==1?new int[]{400,400,600,600}:(seed>>2&3)==2?new int[]{0,0,320,200}:new int[]{-32768,-32768,32767,32767};put(e,0x625e3c,2,clip[0]);put(e,0x622be4,2,clip[1]);put(e,0x498c24,2,clip[2]);put(e,0x498c20,2,clip[3]);for(int tick=0;tick<96;tick++){dx=-999;dy=-999;sprite=-1;height=0;angle=0;transformed=0;if(read(e,O,2)!=(65536-1000))run(e,0x4028a0,O,0);out.println("{\"kind\":\"effectTick\",\"scenario\":"+s+++",\"hash\":"+hash(e.readMemory(toAddr(O),44))+",\"sprite\":"+sprite+",\"x\":"+dx+",\"y\":"+dy+",\"height\":"+height+",\"angle\":"+angle+",\"transformed\":"+transformed+"}");}}
 }finally{e.dispose();}}}
}
