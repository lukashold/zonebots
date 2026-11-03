// Numeric-only403200/401c70/402770 constructors and expiry1/10 contacts.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeDeathEffectOracle extends ZNativeWorldEffectExpiryOracle{
 void init(EmulatorHelper e,int kind,int s,int spr)throws Exception{seed=s;calls=0;e.writeMemory(toAddr(O),new byte[44]);put(e,O+6,2,346);put(e,O+8,2,420);put(e,O+15,1,37);put(e,O+22,1,12);put(e,O+24,1,kind==2?7:s%8);put(e,O+25,1,s&3);put(e,O+26,2,spr);run(e,kind==0?0x401c70:kind==1?0x403200:0x402770,0,0,O);}
 @Override public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{0x460910,0x420200,0x41f2d0,0x402160,0x46e660,0x46e6d0})e.setBreakpoint(toAddr(a));
  for(int kind=0;kind<3;kind++)for(int s=0;s<64;s++){init(e,kind,s,kind==0?1234:kind==1?5739:1903);out.println("{\"kind\":\"init\",\"scenario\":"+(kind*64+s)+",\"calls\":"+calls+",\"hash\":"+hash(e.readMemory(toAddr(O),44))+"}");}
  int[]sprites={0x166b,0x616,0x162f,0x16ba,0x74d,0x14d9,0x1637,0x200,0x14c5,0x735,0x1638,0x1640,0x163c,0,250};int[]codes={0,7,9,42,38,40,39,29,127};int[][]points={{-40,512},{-33,512},{-32,512},{-1,512},{0,0},{16,16},{512,512},{2032,2032},{2048,2048},{32767,32767},{-32768,-32768},{320,180},{160,100},{48,48},{-48,-48},{-49,-49}},projection={{0,0},{160,100},{-160,-100},{32767,-32768}};
  for(int kind=0;kind<2;kind++)for(int s=0;s<256;s++){int spr=kind==0?1234:sprites[s%sprites.length];init(e,kind,s,spr);calls=0;int[]p=points[s%points.length],proj=projection[s/16%4];put(e,O,2,7);put(e,O+2,2,-13);put(e,O+4,2,(s&1)!=0?-1:0);put(e,O+6,2,p[0]-proj[0]);put(e,O+8,2,p[1]-proj[1]);put(e,O+18,2,400-s);put(e,0x4a6520,2,proj[0]);put(e,0x4a6530,2,proj[1]);put(e,0x622cb8,2,s%3*12);put(e,0x622cba,2,s%5*8);put(e,0x498c24,2,320);put(e,0x498c20,2,200);put(e,0x6323a0,1,1);put(e,0x4d18ee,1,s&1);put(e,0x4a6e72,1,0);put(e,0x4a6f42,1,0);put(e,0x4a6f46,1,0);put(e,0x4a6e6f,1,s>>1&1);e.writeMemory(toAddr(0x58aaaa),new byte[16384]);int cell=((((p[1]&65535)*(kind==0?8:8))&(kind==0?0xff80:0xff80))+(p[0]>>4))&65535;if(kind==1)cell=(((p[1]&0xfff0)*8)+(p[0]>>4))&65535;if(cell<16384)put(e,0x58aaaa+cell,1,codes[s%codes.length]);run(e,kind==0?0x401af0:0x401800,O,0);
   out.println("{\"kind\":\"expiry\",\"scenario\":"+(kind*256+s)+",\"hash\":"+hash(e.readMemory(toAddr(O),44))+",\"calls\":"+calls+",\"events\":"+eventCount+",\"eventHash\":"+(eventHash&0xffffffffL)+",\"friendly\":"+read(e,0x4a6e72,1)+",\"suppressed\":"+read(e,0x4a6f42,1)+",\"quiet\":"+read(e,0x4a6e6f,1)+",\"soundGate\":"+read(e,0x4d18ee,1)+"}");
  }
 }finally{e.dispose();}}}
}
