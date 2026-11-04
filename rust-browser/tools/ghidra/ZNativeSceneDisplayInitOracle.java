//Actual40c820 radar constructor and41d280 flag rebuild. No media are emitted.
//Radar terrain/fragment helpers are boundaries; flag OBJECT allocation is
//disabled by its actual4a6e6e guard, as on subsequent capture rebuilds.
import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;import java.io.PrintWriter;
public class ZNativeSceneDisplayInitOracle extends GhidraScript{
 static final long B=0x5c5e38L,CP=0x5c3f0cL,F=0x5a1d5eL,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int hash(EmulatorHelper e,long a,int n){int h=0x811c9dc5;for(byte b:e.readMemory(toAddr(a),n))h=(h^(b&255))*16777619;return h;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{for(long a:new long[]{RET,0x40cc90,0x41f8f0})e.setBreakpoint(toAddr(a));
  for(int s=0;s<256;s++){e.writeMemory(toAddr(B),new byte[88]);put(e,B+10,2,(short)(s*997));put(e,B+6,1,s);put(e,B+21,1,s*7);put(e,S,4,RET);put(e,S+4,4,B);put(e,S+8,4,0x498c78+8*76);e.writeRegister("ESP",S);if(!e.run(toAddr(0x40c820),null,monitor))throw new Exception(e.getLastError());for(int budget=0;e.readRegister("EIP").longValue()!=RET;budget++){if(budget>2)throw new Exception("unexpected ctor calls");long sp=e.readRegister("ESP").longValue(),ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}out.println("r,"+s+","+(short)read(e,B+10,2)+","+read(e,B+6,1)+","+read(e,B+21,1));}
  put(e,0x4a6e6e,1,1);e.writeMemory(toAddr(0x622df9),new byte[]{0,1,2,3,4});for(int s=0;s<512;s++){byte[]flags=new byte[384];for(int j=0;j<flags.length;j++)flags[j]=(byte)(j*17+s);e.writeMemory(toAddr(F),flags);e.writeMemory(toAddr(CP),new byte[64*30]);for(int i=0;i<64;i++){long c=CP+i*30;put(e,c,2,i<s%65?0:-1);put(e,c+24,1,(i+s)%7);put(e,c+8,1,(i+s)%5);put(e,c+18,1,(i*7+s)%64);put(e,c+19,1,(i*13+s)%64);}put(e,S,4,RET);e.writeRegister("ESP",S);if(!e.run(toAddr(0x41d280),null,monitor))throw new Exception(e.getLastError());if(e.readRegister("EIP").longValue()!=RET)throw new Exception("unexpected rebuild call");out.println("f,"+s+","+(hash(e,F,384)&0xffffffffL));}
 }finally{e.dispose();}}}
}
