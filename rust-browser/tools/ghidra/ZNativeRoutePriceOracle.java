// Native route-comparison raster0045cf90, isolated from shortcut selection.
// Uses synthetic terrain and a supplied numeric cost table, no game resources.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRoutePriceOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,P=U+0x1000,C=U+0x2000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[][] starts={{16,16},{31,47},{127,64},{1023,1023}},deltas={{-47,-32},{-32,0},{-17,31},{0,-47},{0,0},{0,63},{16,32},{17,31},{64,-15},{79,48}};int[] prices={2,5,10,12,72};
  for(int mode=0;mode<3;mode++)for(int si=0;si<starts.length;si++)for(int di=0;di<deltas.length;di++)for(int legs=1;legs<3;legs++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x3000]);byte[]map=new byte[16384];for(int y=0;y<128;y++)for(int x=0;x<128;x++)map[y*128+x]=(byte)((x*7+y*11+mode*3)%67);e.writeMemory(toAddr(0x58aaaa),map);byte[]cost=new byte[128];for(int i=0;i<128;i++)cost[i]=(byte)prices[(i+mode)%5];e.writeMemory(toAddr(C),cost);put(e,0x633a14,4,C);put(e,0x4d2be4,2,0);
    int x=starts[si][0],y=starts[si][1],tx=x+deltas[di][0],ty=y+deltas[di][1];put(e,P,2,x);put(e,P+2,2,y);put(e,P+4,2,tx);put(e,P+6,2,ty);int end=8;
    if(legs==2){put(e,P+8,2,x+32);put(e,P+10,2,y+16);end=12;}put(e,P+end,2,65535);put(e,P+end+2,2,65535);
    e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",P);if(!e.run(toAddr(0x45cf90),null,monitor))throw new Exception(e.getLastError());
    out.println("{\"mode\":"+mode+",\"start\":"+si+",\"delta\":"+di+",\"legs\":"+legs+",\"price\":"+e.readRegister("EAX").intValue()+",\"special\":"+(short)read(e,0x4d2be4,2)+"}");
   }finally{e.dispose();}
  }
 }}
}
