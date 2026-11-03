// Actual5a080 leader prefix including real4b590 on empty follower slots.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeLeaderGroupPrefixOracle extends GhidraScript{
 static final long S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<256;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(R),new byte[72]);for(int role=0;role<21;role++)put(e,R+19+role,1,255);
   put(e,U+0x5e,1,new int[]{-1,0,1}[(s>>1)%3]);put(e,U+0x6e,1,new int[]{0,1,254,255}[(s>>3)&3]);put(e,U+0x35,1,new int[]{0,5,8,19,7}[(s>>5)%5]);put(e,R+0x3f,1,new int[]{0,1,4,255}[(s>>2)&3]);put(e,R+0x40,1,new int[]{0,1,127,128,255}[(s>>4)%5]);put(e,R+0x3b,1,(s&1)!=0?255:17);put(e,R+0x10,1,(s&128)!=0?1:0);put(e,R+9,1,(s*7)&255);put(e,0x632194,4,R);
   e.setBreakpoint(toAddr(0x45a18a));e.writeRegister("ESP",S);e.writeRegister("ESI",U);if(!e.run(toAddr(0x45a117),null,monitor))throw new Exception(e.getLastError());
   out.println("{\"scenario\":"+s+",\"counter6e\":"+read(e,U+0x6e,1)+",\"creation\":"+read(e,R+0x3f,1)+",\"hold\":"+read(e,R+0x40,1)+",\"selector\":"+read(e,R+0x3b,1)+",\"flags\":"+read(e,R+9,1)+"}");
  }finally{e.dispose();}
 }}}
}
