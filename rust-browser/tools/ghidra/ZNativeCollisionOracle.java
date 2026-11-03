// Complete native infantry/vehicle contact dispatchers, including diagonals.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeCollisionOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,T=U+0x1000,D=U+0x2000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[]fractions={0,1,7,8,9,15},offsets={-128,-127,-126,0,1,2,128,129,130,256,257,258};
  int[][]directions={{0,-1},{1,-1},{1,0},{1,1},{0,1},{-1,1},{-1,0},{-1,-1}};
  for(int vehicle=0;vehicle<2;vehicle++)for(int fx:fractions)for(int fy:fractions)for(int d=0;d<8;d++)for(int pattern=0;pattern<14;pattern++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x3000]);byte[]map=new byte[16384];int base=32*128+32;
    if(pattern<12)map[base+offsets[pattern]]=(byte)(1+((fx+fy+d)&1));
    if(pattern==13)java.util.Arrays.fill(map,(byte)2);
    e.writeMemory(toAddr(0x58aaaa),map);put(e,T+1,1,255);put(e,T+2,1,1);put(e,U+0x1c,4,T);
    int ax=512+fx,ay=512+fy;int dx=directions[d][0],dy=directions[d][1];
    put(e,U+0x2c,2,ax-dx);put(e,U+0x2e,2,ay-dy);put(e,0x635b64,4,ax);put(e,0x635b68,4,ay);put(e,D,1,dy);put(e,D+1,1,dx);
    put(e,S,4,RET);put(e,S+4,4,D+1);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EDX",D);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(vehicle==1?0x44e3a0:0x45a450),null,monitor))throw new Exception(e.getLastError());
    out.println(vehicle+","+fx+","+fy+","+d+","+pattern+","+(e.readRegister("EAX").intValue()&255)+","+read(e,U+0x2c,2)+","+read(e,U+0x2e,2)+","+(short)read(e,0x633a28,2)+","+(short)read(e,0x633a20,2)+","+read(e,0x4d02b6,1));
   }finally{e.dispose();}
  }
  for(int vehicle=0;vehicle<2;vehicle++)for(int pause:new int[]{1,31,32,33,63,64,255}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x3000]);put(e,U+0xbc,1,pause);put(e,U+0x1c,4,T);put(e,D,1,-1);put(e,D+1,1,1);put(e,0x635b64,4,512);put(e,0x635b68,4,512);put(e,S,4,RET);put(e,S+4,4,D+1);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EDX",D);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(vehicle==1?0x44e3a0:0x45a450),null,monitor))throw new Exception(e.getLastError());
    out.println("pause,"+vehicle+","+pause+","+read(e,U+0xbc,1)+","+(e.readRegister("EAX").intValue()&255)+","+(short)read(e,0x633a28,2)+","+(short)read(e,0x633a20,2)+","+read(e,0x4d02b6,1));
   }finally{e.dispose();}
  }
 }}
}
