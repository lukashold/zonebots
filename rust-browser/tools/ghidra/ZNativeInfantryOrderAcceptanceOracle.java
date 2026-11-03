// Actual53290 successful live-target branch + real48470/52ab0. Spatial range
//49a90, path clearing52500 and path builder4490b0 are explicit recorded inputs.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeInfantryOrderAcceptanceOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,T=U+192,R=0x60ec00L,O=0x612f80L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<256;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[2*192]);e.writeMemory(toAddr(R),new byte[72]);e.writeMemory(toAddr(O),new byte[44]);put(e,0x632194,4,R);put(e,U+0x7c,1,new int[]{0,3,15,16}[s%4]);put(e,U+0x31,1,4|((s&4)!=0?8:0));put(e,U+0x9c,1,s*11);put(e,U+0x71,1,s*7);put(e,U+0x5c,1,5);put(e,U+0x5d,1,3);put(e,U+0x36,2,0);put(e,U+0x50,4,1);put(e,U+0x56,2,s&1);put(e,U+0x2c,2,50+s%10*11);put(e,U+0x2e,2,60+s%7*13);put(e,U+2,1,new int[]{-128,-1,0,40,60,120,127}[s%7]);put(e,T+0x31,1,5);put(e,T+0x7c,1,s%4);put(e,T+0x2c,2,300+s*31);put(e,T+0x2e,2,400+s*17);put(e,R+9,1,s*13);put(e,R+61,1,s*17);put(e,O+13,1,s*13);
   for(long pc:new long[]{RET,0x452500,0x449a90,0x4490b0})e.setBreakpoint(toAddr(pc));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);if(!e.run(toAddr(0x453290),null,monitor))throw new Exception(e.getLastError());int clear=0,route=0,stops=0;while(e.readRegister("EIP").longValue()!=RET){if(++stops>3)throw new Exception("Too many stops");long pc=e.readRegister("EIP").longValue();if(pc==0x452500){clear++;e.writeRegister("EAX",0);}else if(pc==0x449a90)e.writeRegister("EAX",s&1);else if(pc==0x4490b0){route++;e.writeRegister("EAX",0);}else throw new Exception("Unexpected stop");resume(e);}
   out.println("{\"scenario\":"+s+",\"order\":"+(byte)read(e,U+0x5c,1)+",\"flags31\":"+read(e,U+0x31,1)+",\"flags9c\":"+read(e,U+0x9c,1)+",\"flags71\":"+read(e,U+0x71,1)+",\"motion61\":"+read(e,U+0x61,1)+",\"counter68\":"+(byte)read(e,U+0x68,1)+",\"group9\":"+read(e,R+9,1)+",\"group3d\":"+read(e,R+61,1)+",\"object13\":"+read(e,O+13,1)+",\"clear\":"+clear+",\"route\":"+route+",\"result\":"+(e.readRegister("EAX").intValue()&65535)+"}");
  }finally{e.dispose();}
 }}}
}
