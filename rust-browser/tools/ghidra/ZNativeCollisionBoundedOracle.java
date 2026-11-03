// Bounded original segment DDA. Stop before world-aware arrival/contact actions.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeCollisionBoundedOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x633950L,T=0x1001000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int instructions=0;instructions<20000;instructions++){long pc=e.readRegister("EIP").longValue();if(pc==RET||pc==0x451170||pc==0x4518e0)return;if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16)+" for "+Long.toHexString(f));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[][]starts={{400,480},{464,512},{560,300},{600,700},{544,530}};int[]goals={32+32*128,45+30*128,32+15*128,20+45*128};
  for(int vehicle=0;vehicle<2;vehicle++)for(int mode=0;mode<3;mode++)for(int si=0;si<starts.length;si++)for(int gi=0;gi<goals.length;gi++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0xc0]);e.writeMemory(toAddr(T),new byte[0x100]);byte[]map=new byte[16384];for(int y=0;y<128;y++)for(int x=0;x<128;x++)if(mode!=0&&x>=30&&x<=34&&y>=20&&y<=40)map[y*128+x]=(byte)mode;e.writeMemory(toAddr(0x58aaaa),map);put(e,T+1,1,255);put(e,T+2,1,1);put(e,U+0x18,4,T);put(e,U+0x1c,4,T);put(e,U+0x7c,1,vehicle);put(e,U+0x35,1,2);put(e,U+0x31,1,4);put(e,U+0x62,1,1);put(e,U+0x2c,2,starts[si][0]);put(e,U+0x2e,2,starts[si][1]);int gx=(goals[gi]&127)*16+(vehicle==1?7:0),gy=(goals[gi]>>7)*16+(vehicle==1?0:-2);
    e.setBreakpoint(toAddr(RET));put(e,S+4,4,gy);e.writeRegister("EDX",gx);call(e,0x4525a0);int octant=(int)read(e,U+0x39,1);e.setBreakpoint(toAddr(0x451170));e.setBreakpoint(toAddr(0x4518e0));int tick=0;boolean collided=false,arrived=false;int px=starts[si][0],py=starts[si][1],cx=-1,cy=-1,axes=0;
    while(++tick<=4096){call(e,0x4502f0);long pc=e.readRegister("EIP").longValue();if(pc==0x451170){arrived=true;px=(short)read(e,0x635b64,2);py=(short)read(e,0x635b68,2);break;}if(pc==0x4518e0){collided=true;px=(short)read(e,U+0x2c,2);py=(short)read(e,U+0x2e,2);cx=(short)read(e,0x633a28,2);cy=(short)read(e,0x633a20,2);axes=(int)read(e,0x4d02b6,1);if(vehicle==1){px=(short)read(e,0x635b64,2);py=(short)read(e,0x635b68,2);}break;}}
    if(!arrived&&!collided)throw new Exception("DDA budget exceeded");out.println(vehicle+","+mode+","+si+","+gi+","+(collided?1:0)+","+px+","+py+","+cx+","+cy+","+axes+","+octant+","+tick);out.flush();
   }finally{e.dispose();}
  }
 }}
}
