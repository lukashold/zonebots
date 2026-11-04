//Numeric55f10 BUILD/fort clip selection, regular tank layers and454630 raster
//scissors. Executes original query/composition/dispatch; texture, palette
//upload and GPU-batch endpoints are boundaries. Records are synthetic and
//contain no original pixels. Each fixture starts with clip-Y scratchword0;
//disabled masks normalize irrelevant retained X/Y/height in the trace.
import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;import java.io.PrintWriter;
public class ZNativeHardwareClipOracle extends GhidraScript{
 static final long U=0x5e30c8L,H=0x72000000L,Q=0x62dc50L,E=0x628e18L,B=0x5c5e38L,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int mix(int h,int...v){for(int n:v)h=(h^n)*16777619;return h;}
 String hex(EmulatorHelper e,long a,int n){if(n==0)return "";StringBuilder s=new StringBuilder();for(byte b:e.readMemory(toAddr(a),n))s.append(String.format("%02x",b&255));return s.toString();}
 int floatInt(EmulatorHelper e,long a){return (int)Float.intBitsToFloat((int)read(e,a,4));}
 void call(EmulatorHelper e,long pc)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);if(!e.run(toAddr(pc),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter o=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(int k=0;k<9;k++)o.println("TABLE,"+k+","+(short)read(e,0x498cb8+k*76,2)+","+(short)read(e,0x498cba+k*76,2));
  for(long a:new long[]{RET,0x46bde0,0x465cb0,0x466090,0x466180,0x466ac0})e.setBreakpoint(toAddr(a));put(e,0x622cb8,2,40);put(e,0x622cba,2,60);put(e,0x498c24,2,1024);put(e,0x498c20,2,1024);put(e,0x4d02df,1,0);put(e,0x680cfc,4,0);put(e,0x4d18ff,1,0);
  for(int s=0;s<8192;s++){int family=new int[]{6,25,30}[s%3],phase=new int[]{9,30,31,44,62,63,64,65,66}[s/3%9],terrain=new int[]{0,30,31,44,62,63,64,65,66}[s/27%9],mode=s/243%12,kind=mode<9?mode:-1,bx=500-new int[]{0,16,79,80,81}[s/5%5],by=600-new int[]{-1,0,1,16,48,64,65,96}[s/7%8],hp=new int[]{0,1,-1}[s/11%3],allow=s/13%2,slot=s/17%2==0?0:17,second=s/19%2,heading=s/23%8,turret=s/29%8,frame=s/31%3,body=new int[]{0,1,2,255}[s/37%4],hatch=new int[]{-1,0,1,2,3,4}[s/41%6],flags=new int[]{0,1,2,3,4,5,6,7}[s/43%8],mask=new int[]{0,1,3,7}[s/47%4],elevation=new int[]{-32,0,7}[s/53%3];
   e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(H),new byte[44]);e.writeMemory(toAddr(Q),new byte[32]);e.writeMemory(toAddr(E),new byte[256]);e.writeMemory(toAddr(B),new byte[35*88]);for(int k=0;k<35;k++)put(e,B+k*88+4,1,255);
   if(kind>=0){long p=B+slot*88;put(e,p,2,bx);put(e,p+2,2,by);put(e,p+4,1,kind);put(e,p+30,2,hp);}if(second!=0){long p=B+34*88;put(e,p,2,480);put(e,p+2,2,580);put(e,p+4,1,1);put(e,p+30,2,1);}
   put(e,0x622d54,2,mode==9?500:2000);put(e,0x622d56,2,mode==9?600:2000);put(e,0x622d58,2,mode==10?500:3000);put(e,0x622d5a,2,mode==10?600:3000);put(e,0x4a69f7,1,allow);put(e,0x58aaaa,1,terrain);
   put(e,U+8,4,H);put(e,U+44,2,500);put(e,U+46,2,600);put(e,U+48,1,family);put(e,U+50,2,frame);put(e,U+53,1,phase);put(e,U+57,1,heading);put(e,U+100,1,flags);put(e,U+105,1,4);put(e,U+118,1,elevation);put(e,U+124,1,1);put(e,U+132,2,-1);put(e,U+151,1,27);put(e,U+152,1,mask);put(e,H+4,1,body);put(e,H+8,2,24);put(e,H+10,2,100);put(e,H+21,1,turret);put(e,H+25,1,hatch);put(e,0x62b3a4,4,Q);put(e,0x62d5d8,4,E);put(e,0x477063,1,0);put(e,0x47717f,1,0);put(e,0x49cc84,4,0xbf800000L);put(e,0x4d1912,2,0);call(e,0x455f10);if(e.readRegister("EIP").longValue()!=RET)throw new Exception("producer boundary");int q=(int)(read(e,0x62b3a4,4)-Q),n=(int)(read(e,0x62d5d8,4)-E);if(q!=8||n<0||n>128||n%16!=0)throw new Exception("queue shape");String header=hex(e,Q,q),layers=hex(e,E,n);int clip=(short)read(e,0x4d1910,2),line=(short)read(e,0x4d1912,2),fa=(int)read(e,U+100,1),ma=(int)read(e,U+152,1),trace=0x811c9dc5,draws=0;call(e,0x4549f0);
   for(int budget=0;e.readRegister("EIP").longValue()!=RET;budget++){if(budget>40)throw new Exception("raster budget");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();if(pc==0x465cb0)e.writeRegister("ECX",0);else if(pc==0x466090||pc==0x466180){draws++;trace=mix(trace,(short)e.readRegister("ECX").intValue(),(short)e.readRegister("EDX").intValue(),(short)read(e,sp+8,2),(int)read(e,0x477063,1),floatInt(e,0x49cc84)<0?0:floatInt(e,0x49cc7c),floatInt(e,0x49cc84)<0?0:floatInt(e,0x49cc80),floatInt(e,0x49cc84),floatInt(e,0x49cc84)<0?0:floatInt(e,0x49cc88));}else if(pc!=0x46bde0)throw new Exception("raster boundary "+Long.toHexString(pc));long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
   o.println(s+","+family+","+phase+","+terrain+","+mode+","+kind+","+bx+","+by+","+hp+","+allow+","+slot+","+second+","+heading+","+turret+","+frame+","+body+","+hatch+","+flags+","+mask+","+elevation+","+header+","+layers+","+clip+","+(clip==0?0:line)+","+fa+","+ma+","+(trace&0xffffffffL)+","+draws+","+floatInt(e,0x49cc84));
  }
 }finally{e.dispose();}}}
}
