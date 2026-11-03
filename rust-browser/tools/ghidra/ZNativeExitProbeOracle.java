//Complete457b60 footprints and59830's clockwise search with the real41c600.
//59720 is a tail boundary: this oracle verifies availability, not unloading.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.util.*;
public class ZNativeExitProbeOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,MAP=0x58aaaaL,O=0x612f80L,M=0x1200000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int terrainCalls,objectCalls;
 boolean call(EmulatorHelper e,long f,long cx,long dx,int argument,boolean tail)throws Exception{
  put(e,S,4,RET);put(e,S+4,4,argument);e.writeRegister("ESP",S);e.writeRegister("ECX",cx);e.writeRegister("EDX",dx);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);terrainCalls=0;objectCalls=0;
  for(int n=0;n<100000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET)return false;if(tail&&pc==0x459720)return true;if(pc==0x457b60)terrainCalls++;if(pc==0x41c600)objectCalls++;if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int world=0;world<2;world++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{
   put(e,0x622dc4,1,0);put(e,0x622c7a,2,0);put(e,0x625eae,1,world==1?3:0);call(e,0x433b30,U,0,0,false);e.writeMemory(toAddr(U),new byte[192]);byte[]map=new byte[16384];Arrays.fill(map,(byte)10);e.writeMemory(toAddr(MAP),map);int[]offsets=new int[8];for(int d=0;d<8;d++)offsets[d]=(short)read(e,0x497094+d*2,2);int[]probes={0,1,128,129,2,130};int previous=-1;
   for(int direction=0;direction<8;direction++)for(int probe=0;probe<6;probe++)for(int code=0;code<128;code++)for(int high=0;high<2;high++){
    if(previous>=0)put(e,MAP+previous,1,10);int cell=4128+offsets[direction]+probes[probe];put(e,MAP+cell,1,code|(high<<7));previous=cell;call(e,0x457b60,0,4128+offsets[direction],direction,false);out.println("foot,"+world+","+direction+","+probe+","+code+","+high+","+(e.readRegister("EAX").intValue()&255));
   }
   for(int scenario=0;scenario<128;scenario++)for(int start=0;start<8;start++){
    map=new byte[16384];Arrays.fill(map,(byte)10);int x=500+(scenario&15),y=600+((scenario>>3)&15),base=((y&0xfff0)*8+(x>>4))&0xffff;int pattern=scenario%8;
    if(pattern!=0)for(int row=-3;row<=4;row++)for(int col=-3;col<=4;col++){int n=(scenario*17+row*11+col*7)&31;if(n<pattern)map[(base+row*128+col)&16383]=(byte)new int[]{9,31,32,40,38,62,63}[n%7];}
    e.writeMemory(toAddr(MAP),map);e.writeMemory(toAddr(O),new byte[200*44]);for(int i=0;i<200;i++)put(e,O+i*44,2,-1);
    int[]deltas={-25,-24,-16,-8,-1,0,1,8,16,24,25};for(int i=0;i<4;i++){long obj=O+i*44,meta=M+i*26;int dx=deltas[(scenario+i*3)%11],dy=deltas[(scenario/3+i*5)%11];put(e,obj,2,x+dx);put(e,obj+2,2,y+dy);put(e,obj+4,4,meta);put(e,obj+13,1,new int[]{0,16,64,80}[(scenario+i)%4]);put(e,meta+18,1,scenario*13+i*17);put(e,meta+20,1,scenario*43+i*7);}
    put(e,U+0x2c,2,x);put(e,U+0x2e,2,y);put(e,U+0x39,1,start);put(e,0x4a6e6a,1,(scenario>>5)&1);put(e,0x4a6e6b,1,(scenario>>6)&1);boolean chosen=call(e,0x459830,U,0,0,true);out.println("exit,"+world+","+scenario+","+start+","+(chosen?1:0)+","+read(e,U+0x39,1)+","+terrainCalls+","+objectCalls+","+read(e,0x4a6e6b,1));
   }
   out.flush();
  }finally{e.dispose();}
 }}}
}
