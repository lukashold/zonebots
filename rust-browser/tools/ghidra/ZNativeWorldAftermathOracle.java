// Numeric tests of bridge terrain and inclusive collapse selectors.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeWorldAftermathOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,O=0x612f80L,R=0x6151e0L,T=0x58aaaaL,G=0x5b6da4L;
 int killed;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void run(EmulatorHelper e,long address,long ecx,long edx,long...args)throws Exception{
  killed=0;e.writeRegister("ESP",S);put(e,S,4,RET);for(int i=0;i<args.length;i++)put(e,S+4+i*4,4,args[i]);e.writeRegister("ECX",ecx);e.writeRegister("EDX",edx);e.setBreakpoint(toAddr(RET));
  if(!e.run(toAddr(address),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),stack=e.readRegister("ESP").longValue();if(pc!=0x41e6e0)throw new Exception("Unexpected "+Long.toHexString(pc));int slot=(int)((e.readRegister("ECX").longValue()-U)/0xc0);killed|=1<<slot;long ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
 }
 long hash(byte[] bytes){int h=0x811c9dc5;for(byte b:bytes)h=(h^(b&255))*16777619;return h&0xffffffffL;}
 void terrain(EmulatorHelper e,int seed)throws Exception{byte[]plane=new byte[16384];for(int i=0;i<plane.length;i++)plane[i]=(byte)(i*13+seed*37);e.writeMemory(toAddr(T),plane);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  EmulatorHelper e=new EmulatorHelper(currentProgram);
  try{for(int s=0;s<48;s++){terrain(e,s);int orientation=s&1,length=2+s/2%12;put(e,R+4,2,orientation==0?length*16:48);put(e,R+6,2,orientation!=0?length*16:48);put(e,R+12,2,orientation);run(e,0x4094e0,R,500,500);out.println("{\"kind\":\"bridgeTerrain\",\"scenario\":"+s+",\"value\":"+(e.readRegister("EAX").intValue()&255)+",\"hash\":"+hash(e.readMemory(toAddr(T),16384))+"}");}}finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);e.setBreakpoint(toAddr(0x41e6e0));
  try{for(int s=0;s<256;s++){e.writeMemory(toAddr(U),new byte[240*0xc0]);put(e,0x4d02d7,1,s&1);for(int i=0;i<4;i++){long p=U+i*0xc0;put(e,p+0x2c,2,500+(s*13+i*7)%65-32);put(e,p+0x2e,2,500+(s*17+i*11)%65-32);put(e,p+0x31,1,(s>>(i+1)&1)*4);put(e,p+0x7c,1,(s+i)%4);}run(e,0x459d10,484,484,516,516);out.println("{\"kind\":\"unitCollapse\",\"scenario\":"+s+",\"mask\":"+killed+"}");}}finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);
  try{for(int s=0;s<256;s++){terrain(e,s);e.writeMemory(toAddr(O),new byte[200*44]);for(int i=0;i<200;i++)put(e,O+i*44,2,65535);for(int i=0;i<4;i++){long p=O+i*44;put(e,p,2,500+(s*13+i*7)%65-32);put(e,p+2,2,500+(s*17+i*11)%65-32);put(e,p+4,4,0x498f38+((s+i)%39)*26);for(int j=0;j<4;j++)put(e,p+20+j,1,s*19+i*37+j*11);}run(e,0x43f650,484,484,516,516);int mask=0;for(int i=0;i<4;i++)if(read(e,O+i*44,2)==65535)mask|=1<<i;out.println("{\"kind\":\"objectCollapse\",\"scenario\":"+s+",\"mask\":"+mask+",\"hash\":"+hash(e.readMemory(toAddr(T),16384))+"}");}}finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);
  try{for(int s=0;s<32;s++){terrain(e,s);e.writeMemory(toAddr(G),new byte[96*0x88]);int[]rect={480,480,528+(s%4)*16,528+(s/4%4)*16};for(int j=0;j<4;j++)put(e,G+j*2,2,rect[j]);run(e,0x409690,R,500,500);out.println("{\"kind\":\"roadCollapse\",\"scenario\":"+s+",\"left\":"+read(e,G,2)+",\"hash\":"+hash(e.readMemory(toAddr(T),16384))+"}");}}finally{e.dispose();}
 }}
}
