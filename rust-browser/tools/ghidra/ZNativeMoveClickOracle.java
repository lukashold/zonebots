// Real4416b0 branch sequencing. UI geometry lookups/renderer feedback and
//45ef00 are explicit recorded boundaries; no source branch is substituted.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeMoveClickOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,A=0x70000000L,F=A+64,O=A+128;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<256;s++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  e.writeMemory(toAddr(U),new byte[192]);put(e,U+0x30,1,4);put(e,U+0x5e,1,0);put(e,U+0x7c,1,s%4==0?1:0);put(e,U+0x2c,2,s%3==0?500:50);put(e,U+0x2e,2,s%3==0?600:60);put(e,A,4,U);put(e,0x622d1a,2,1);put(e,0x622cc0,4,1);put(e,0x4d0e32,1,s&1);put(e,F,2,700);put(e,F+2,2,800);put(e,O,2,500);put(e,O+2,2,600);put(e,0x632398,4,F);
  long[]pcs={RET,0x438950,0x41cbd0,0x41c600,0x41c290,0x4387b0,0x437790,0x438740,0x45ef00,0x441550};for(long p:pcs)e.setBreakpoint(toAddr(p));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",A);if(!e.run(toAddr(0x4416b0),null,monitor))throw new Exception(e.getLastError());int event=0,x=-999,y=-999,q=0,terrain=0,ax=-999,ay=-999,stops=0;
  while(e.readRegister("EIP").longValue()!=RET){if(++stops>20)throw new Exception("too many stops");long pc=e.readRegister("EIP").longValue(),cx=e.readRegister("ECX").longValue(),dx=e.readRegister("EDX").longValue(),sp=e.readRegister("ESP").longValue();int result=0;
   if(pc==0x438950){put(e,cx,2,200+s);put(e,dx,2,300+s);}else if(pc==0x41cbd0){q=q*10+1;result=(s&2)!=0?1:0;}else if(pc==0x41c600){q=q*10+2;ax=(short)cx;ay=(short)dx;put(e,read(e,sp+4,4),4,O);put(e,read(e,sp+8,4),4,0x498f38+31*26);result=(s&4)!=0?1:0;}else if(pc==0x41c290){q=q*10+3;result=(s&8)!=0?1:0;put(e,cx,2,read(e,cx,2)+3);put(e,dx,2,read(e,dx,2)-5);}else if(pc==0x4387b0){q=q*10+4;result=(s&16)!=0?1:0;put(e,cx,2,read(e,cx,2)-7);put(e,dx,2,read(e,dx,2)+11);}else if(pc==0x437790){terrain++;result=(s&32)!=0?1:0;}else if(pc==0x45ef00){event++;x=(short)read(e,0x62dc00,2);y=(short)read(e,0x62dc3c,2);}e.writeRegister("EAX",result);resume(e);
  }
  out.println("{\"scenario\":"+s+",\"event\":"+event+",\"x\":"+x+",\"y\":"+y+",\"queries\":"+q+",\"terrain\":"+terrain+",\"apcX\":"+ax+",\"apcY\":"+ay+"}");
 }finally{e.dispose();}}}}
}
