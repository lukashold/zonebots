// Real19340→437790/437830 on a full native terrain plane. RNG result and
//19240 occupied-goal predicate are explicit inputs; no Web geometry is used.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeSectorApproachOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,P=0x58aaaaL,T=0x4d2000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{byte[]plane=new byte[16384];for(int i=0;i<plane.length;i++){int x=i&127,y=i>>7;plane[i]=(byte)(((x*3+y*7+s)%17<(s%5))?1:0);}e.writeMemory(toAddr(P),plane);e.writeMemory(toAddr(T),new byte[128]);put(e,T+1,1,255);e.writeMemory(toAddr(U),new byte[192]);put(e,U+0x18,4,T);put(e,U+0x30,1,new int[]{4,6,25,30,24,32,34,36,38,12}[s%10]);put(e,0x62d5d4,2,512+s%16);put(e,0x62d5dc,2,768+s/16%16);put(e,0x62dc00,2,-123);put(e,0x62dc3c,2,456);
   for(long a:new long[]{RET,0x460910,0x419240})e.setBreakpoint(toAddr(a));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);if(!e.run(toAddr(0x419340),null,monitor))throw new Exception(e.getLastError());int random=0,checks=0,stops=0;
   while(e.readRegister("EIP").longValue()!=RET){if(++stops>10)throw new Exception("Too many stops");long pc=e.readRegister("EIP").longValue();if(pc==0x460910){random++;e.writeRegister("EAX",s*31);}else if(pc==0x419240){checks++;int x=(short)e.readRegister("EDX").longValue(),y=(short)read(e,e.readRegister("ESP").longValue()+4,2);e.writeRegister("EAX",((x>>4)+(y>>4)+s)%5==0?1:0);}else throw new Exception("Unexpected stop");resume(e);}
   out.println("{\"scenario\":"+s+",\"failed\":"+(e.readRegister("EAX").intValue()&255)+",\"x\":"+(short)read(e,0x62dc00,2)+",\"y\":"+(short)read(e,0x62dc3c,2)+",\"random\":"+random+",\"checks\":"+checks+"}");
  }finally{e.dispose();}
 }}}
}
