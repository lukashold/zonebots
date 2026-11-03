// Real578b0, including the real57790/41c8b0 APC group-target predicate.
// Only final path search59c60 and path start52810 are external success inputs.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeHardwareClaimOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L,O=0x612f80L+17*44;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{
   e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(R),new byte[72]);e.writeMemory(toAddr(0x612f80),new byte[200*44]);e.writeMemory(toAddr(0x5a1d5e),new byte[64*6]);for(int i=0;i<64;i++)put(e,0x5a1d5e+i*6,2,65535);
   int type=new int[]{5,8,21,22,23,24,26,29,31,33,35,37,25}[s%13],flags=5|((s&8)!=0?8:0)|((s&16)!=0?32:0)|((s&32)!=0?64:0);
   put(e,U+0x31,1,flags);put(e,U+0x64,1,(s&128)!=0?8:0);put(e,U+0xaa,1,(s&64)!=0?1:0);put(e,U+0x30,1,4);put(e,U+0x5e,1,(s&2)!=0?1:255);put(e,U+0x5f,1,0);put(e,R+58,1,(s&4)!=0?0:1);put(e,R+44,2,(s&256)!=0?1000:308);put(e,R+46,2,(s&256)!=0?1000:408);put(e,0x632194,4,R);
   put(e,O,2,300);put(e,O+2,2,400);put(e,O+4,4,0x498f38+type*26);put(e,O+13,1,0x70|((s&1)!=0?1:0));put(e,0x4a6e6a,1,0);
   e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);e.writeRegister("EDX",17);for(long a:new long[]{RET,0x459c60,0x452810})e.setBreakpoint(toAddr(a));if(!e.run(toAddr(0x4578b0),null,monitor))throw new Exception(e.getLastError());
   int routed=0,x=0,y=0;while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue();if(pc==0x459c60){routed++;x=(short)e.readRegister("EDX").intValue();y=(short)read(e,e.readRegister("ESP").longValue()+4,4);}else if(pc!=0x452810)throw new Exception("Unexpected claim stop");e.writeRegister("EAX",0);resume(e);}
   out.println("{\"scenario\":"+s+",\"routed\":"+routed+",\"x\":"+x+",\"y\":"+y+",\"flags31\":"+read(e,U+0x31,1)+",\"objectFlags\":"+read(e,O+13,1)+",\"contact\":"+read(e,U+0x5d,1)+",\"target\":"+read(e,U+0x36,2)+",\"bonus\":"+read(e,U+0x8c,1)+"}");
  }finally{e.dispose();}
 }}}
}
