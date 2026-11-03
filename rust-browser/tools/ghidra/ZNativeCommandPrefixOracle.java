// Actual54160 with real54070/540b0. Transport cleanup59d80, follower-fire
//cleanup54100 and final per-opcode handlers are explicit recorded boundaries.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeCommandPrefixOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[3*192]);e.writeMemory(toAddr(R),new byte[72]);for(int i=0;i<21;i++)put(e,R+19+i,1,i<3?i:255);for(int i=0;i<3;i++)put(e,U+i*192+0x8a,1,s*(i+3)+i*17);put(e,0x632194,4,R);put(e,0x4a651e,1,s&1);put(e,U+0x5e,1,new int[]{-1,0,1}[s%3]);put(e,U+0x5c,1,s%31-15);put(e,U+0x69,1,s*7);put(e,U+0x9c,1,s*11);put(e,U+0xb7,1,s*13);put(e,U+0xaa,1,s*17);put(e,U+0x64,1,s*19);put(e,U+0x50,4,0x1234abcdL+s);put(e,U+0x54,2,-600-s);put(e,U+0x56,2,800+s);put(e,U+0x30,1,(s&2)!=0?4:6);put(e,R+58,1,s%15);put(e,R+44,2,-32700+s*23);put(e,R+46,2,32700-s*17);put(e,R+48,2,-12345+s*31);put(e,R+9,1,s*23);
   long[]calls={0x459d80,0x454100,0x453730,0x452b70,0x452d20,0x452d90,0x453290,0x453b80,0x453ff0,0x453130};for(long a:calls)e.setBreakpoint(toAddr(a));e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);if(!e.run(toAddr(0x454160),null,monitor))throw new Exception(e.getLastError());int cleanup=0,handler=0,n=0;
   while(e.readRegister("EIP").longValue()!=RET){if(++n>3)throw new Exception("Too many stops");long pc=e.readRegister("EIP").longValue();if(pc==calls[0]||pc==calls[1])cleanup++;else{int j=2;while(j<calls.length&&calls[j]!=pc)j++;if(j==calls.length)throw new Exception("Unexpected stop");handler=new int[]{1,2,3,4,5,12,12,13}[j-2];}e.writeRegister("EAX",0);resume(e);}
   out.println("{\"scenario\":"+s+",\"order\":"+(byte)read(e,U+0x5c,1)+",\"flags9c\":"+read(e,U+0x9c,1)+",\"flagsb7\":"+read(e,U+0xb7,1)+",\"protection\":"+read(e,U+0xaa,1)+",\"flags64\":"+read(e,U+0x64,1)+",\"flags69\":"+read(e,U+0x69,1)+",\"x\":"+(int)read(e,U+0x50,4)+",\"y\":"+(short)read(e,U+0x54,2)+",\"aux\":"+(short)read(e,U+0x56,2)+",\"group9\":"+read(e,R+9,1)+",\"cleanup\":"+cleanup+",\"handler\":"+handler+"}");
  }finally{e.dispose();}
 }}}
}
