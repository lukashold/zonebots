// Complete58620, no call boundaries or source stubs.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeOrderSyncOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(R),new byte[72]);put(e,0x632194,4,R);put(e,U+0x5e,1,new int[]{-1,0,1}[s%3]);put(e,U+0x5c,1,s%14);put(e,U+0x69,1,s*7);put(e,U+0x50,4,0x1234abcdL+s);put(e,U+0x54,2,-600-s);put(e,U+0x56,2,800+s);put(e,R+58,1,s%15);put(e,R+44,2,-32700+s*23);put(e,R+46,2,32700-s*17);put(e,R+48,2,-12345+s*31);
   e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);if(!e.run(toAddr(0x458620),null,monitor))throw new Exception(e.getLastError());out.println("{\"scenario\":"+s+",\"order\":"+(byte)read(e,U+0x5c,1)+",\"x\":"+(int)read(e,U+0x50,4)+",\"y\":"+(short)read(e,U+0x54,2)+",\"aux\":"+(short)read(e,U+0x56,2)+",\"flags69\":"+read(e,U+0x69,1)+"}");
  }finally{e.dispose();}
 }}}
}
