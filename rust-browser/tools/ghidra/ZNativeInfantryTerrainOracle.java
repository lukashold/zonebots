//Synthetic records through the original terrain submission prefix.
//RNG values, actor removal and ambient allocation are explicit leaf boundaries.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeInfantryTerrainOracle extends GhidraScript {
 static final long U=0x5e30c8L,Q=0x62dc50L,E=0x628e18L,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[] b=e.readMemory(toAddr(a),n);long v=0;for(int j=0;j<n;j++)v|=(b[j]&255L)<<(8*j);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 String hex(EmulatorHelper e,long a,int n){StringBuilder b=new StringBuilder();for(byte v:e.readMemory(toAddr(a),n))b.append(String.format("%02x",v&255));return b.toString();}
 long random(int s,int call){if(call==0)return new long[]{0,1,1024,4096,63,64,65535,0xffff0001L}[s/11%8];return ((s*7919L)^(call*2027L)^0x7731a5c9L)&0xffffffffL;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0]);PrintWriter raster=new PrintWriter(getScriptArgs()[0]+".raster.csv")){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long pc:new long[]{RET,0x460910,0x458ec0,0x405c60,0x46bde0,0x465cb0,0x466090,0x466180,0x466ac0})e.setBreakpoint(toAddr(pc));
  for(int s=0;s<8192;s++){
   int code=new int[]{0,9,38,39,42,137,166,167}[s%8],counter=new int[]{-128,-17,-2,-1,0,1,3,6,7,10,11,14,15,16,17,127}[s/8%16],phase=new int[]{0,1,2,3,19,23,42,65}[s/128%8],world=s/1024%5,mp=s/37%2,skip=s/41%2,family=s/43%7==0?6:4,heading=s/47%8,flags=new int[]{0,1,4,5,16,21}[s/53%6],frame=s/59%3,extraIndex=s/61%4,oldX=(short)(s*997),terrainMode=s/67%3;
   int[] pos=new int[][]{{500,600},{-33,-49},{7,8},{32760,-32768},{8,9}}[s/71%5];int cameraX=s/73%2==0?40:pos[0]+33,cameraY=s/79%2==0?60:pos[1]+49;
   e.writeMemory(toAddr(U),new byte[192]);byte[] plane=new byte[32768];for(int j=0;j<plane.length;j++)plane[j]=(byte)(terrainMode==0?9:terrainMode==1?0:((j%3==0)?9:38));e.writeMemory(toAddr(0x58aaaaL-16384),plane);
   int cell=(short)((((short)(pos[1]+11)<<3)&0xff80)+((short)(pos[0]+5)>>4));put(e,0x58aaaaL+cell,1,code);
   put(e,U+44,2,pos[0]);put(e,U+46,2,pos[1]);put(e,U+48,1,family);put(e,U+50,2,frame);put(e,U+53,1,phase);put(e,U+57,1,heading);put(e,U+100,1,flags);put(e,U+151,1,counter);put(e,U+130,2,55);put(e,U+132,2,66);put(e,U+136,2,77);put(e,U+126,2,0x3412);put(e,U+128,1,88);
   put(e,0x622cb8,2,cameraX);put(e,0x622cba,2,cameraY);put(e,0x498c24,2,1024);put(e,0x498c20,2,1024);put(e,0x4a653b,1,skip);put(e,0x622dc4,1,mp);put(e,0x622c7a,2,0);put(e,0x622cbc,2,1);put(e,0x625eae,1,world);put(e,0x625eae+240,1,(world+2)%5);put(e,0x4d029f,1,0);
   byte[] old=new byte[16];for(int j=0;j<16;j++)old[j]=(byte)(s*13+j*17);long extra=E+16*extraIndex;e.writeMemory(toAddr(extra),old);e.writeMemory(toAddr(Q),new byte[8]);put(e,Q,2,oldX);put(e,0x62b3a4,4,Q);put(e,0x62d5d8,4,extra);put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);
   if(!e.run(toAddr(0x454d90),null,monitor))throw new Exception(e.getLastError());int calls=0,remove=0,spawn=0;String args="";
   for(int budget=0;e.readRegister("EIP").longValue()!=RET;budget++){
    if(budget>8)throw new Exception("budget");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();
    if(pc==0x460910)e.writeRegister("EAX",random(s,calls++));else if(pc==0x458ec0){remove++;if(read(e,0x4d029f,1)!=1)throw new Exception("forced flag");}else if(pc==0x405c60){spawn++;args=(short)e.readRegister("ECX").intValue()+":"+(short)e.readRegister("EDX").intValue();for(int j=0;j<9;j++)args+=":"+(int)read(e,sp+4+j*4,4);}else throw new Exception("boundary "+Long.toHexString(pc));
    long next=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(next),null,monitor))throw new Exception(e.getLastError());
   }
   int q=(int)(read(e,0x62b3a4,4)-Q),n=(int)(read(e,0x62d5d8,4)-extra);
   int result=e.readRegister("EAX").intValue()&255;
   if(q!=0){
    put(e,0x680cfc,4,0);put(e,0x4d18ff,1,0);put(e,0x477063,1,0);put(e,0x47717f,1,0);put(e,0x49cc84,4,0xbf800000L);put(e,S,4,RET);e.writeRegister("ESP",S);
    if(!e.run(toAddr(0x4549f0),null,monitor))throw new Exception(e.getLastError());
    for(int budget=0;e.readRegister("EIP").longValue()!=RET;budget++){
     if(budget>10)throw new Exception("rasterbudget");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();
     if(pc==0x465cb0)e.writeRegister("ECX",0);else if(pc==0x466090||pc==0x466180){raster.println(s+","+(short)e.readRegister("ECX").intValue()+","+(short)e.readRegister("EDX").intValue()+","+(short)read(e,sp+8,2)+","+read(e,0x477063,1)+","+(int)Float.intBitsToFloat((int)read(e,0x49cc7c,4))+","+(int)Float.intBitsToFloat((int)read(e,0x49cc80,4))+","+(int)Float.intBitsToFloat((int)read(e,0x49cc84,4))+","+(int)Float.intBitsToFloat((int)read(e,0x49cc88,4)));}else if(pc!=0x46bde0)throw new Exception("rasterboundary"+Long.toHexString(pc));
     long next=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(next),null,monitor))throw new Exception(e.getLastError());
    }
   }
   out.println(s+","+code+","+counter+","+phase+","+world+","+mp+","+skip+","+family+","+heading+","+flags+","+frame+","+extraIndex+","+oldX+","+terrainMode+","+pos[0]+","+pos[1]+","+cameraX+","+cameraY+","+result+","+(byte)read(e,U+151,1)+","+read(e,U+53,1)+","+(short)read(e,U+50,2)+","+(short)read(e,U+130,2)+","+(short)read(e,U+132,2)+","+read(e,U+136,2)+","+read(e,U+126,2)+","+read(e,U+128,1)+","+read(e,U+100,1)+","+calls+","+remove+","+spawn+","+q+","+n+","+hex(e,Q,8)+","+hex(e,extra,16)+","+args);
  }
 }finally{e.dispose();}}}
}
