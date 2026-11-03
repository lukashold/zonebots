// Native43c1b0 plus real reverse dummy walks. Fixed CPU ceiling for diagnostics.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRouteSimplifyOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,T=U+0x1000,P=U+0x2000,V=0x633950;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[][]points={{2461,2467,4003},{5277,5283,4259},{2461,2467,5283,5277,4253},{4126,4127,4128,4129,4130,4131},{2461,2467,4003,4136}};
  for(int vehicle=0;vehicle<2;vehicle++)for(int mode=0;mode<2;mode++)for(int fixture=0;fixture<points.length;fixture++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[0x3000]);e.writeMemory(toAddr(V),new byte[0xc0]);byte[]map=new byte[16384];for(int y=0;y<128;y++)for(int x=0;x<128;x++)if(mode==1&&x>=30&&x<=34&&y>=20&&y<=40)map[y*128+x]=1;e.writeMemory(toAddr(0x58aaaa),map);put(e,T+1,1,255);put(e,U+0x18,4,T);put(e,U+0x1c,4,T);put(e,U+0x7c,1,vehicle);put(e,U+0x31,1,4);put(e,U+0x2c,2,464);put(e,U+0x2e,2,512);put(e,0x63288c,4,U);put(e,P+1,1,-points[fixture].length);for(int i=0;i<points[fixture].length;i++)put(e,P+2+i*2,2,points[fixture][i]);put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EDX",P);e.writeRegister("EIP",0x43c1b0);e.getEmulator().setHalt(false);int instructions=0;for(;instructions<1000000;instructions++){if(e.readRegister("EIP").longValue()==RET)break;if(!e.step(monitor))throw new Exception(e.getLastError());}if(instructions==1000000){out.println("BUDGET,"+vehicle+","+mode+","+fixture+","+e.readRegister("EIP").toString(16));out.flush();continue;}int count=-(byte)read(e,P+1,1);StringBuilder line=new StringBuilder(vehicle+","+mode+","+fixture+","+count);for(int i=0;i<count;i++)line.append(',').append((short)read(e,P+2+i*2,2));out.println(line);out.flush();}finally{e.dispose();}
  }
 }}
}
