// Run bounded, actual instruction blocks for the authored/current formation
// writers. Entry gates/path search are explicit external inputs, never stubbed.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeFormationWritersOracle extends GhidraScript{
 static final long U=0x5e30c8L,R=0x60ec00L,D=0x72000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void block(EmulatorHelper e,long start,long end)throws Exception{e.setBreakpoint(toAddr(end));if(!e.run(toAddr(start),null,monitor))throw new Exception(e.getLastError());e.clearBreakpoint(toAddr(end));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int s=0;s<256;s++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    long g=R+120*72;byte[]r=new byte[72];for(int j=0;j<72;j++)r[j]=(byte)(s*7+j);e.writeMemory(toAddr(g),r);e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(D),new byte[82]);
    int layout=s%5,alternate=s&1,flags71=(s&2)!=0?4:0,flags64=(s&4)!=0?128:0;int[]terrains={0,23,151,35};int terrain=terrains[(s>>3)&3];int[]roles={-1,0,1,20};int role=roles[(s>>5)&3];
    put(e,D+1,1,layout);put(e,D+0x48,1,s+17);put(e,0x584963,1,alternate);put(e,0x622cd4,4,g);e.writeRegister("ESI",D);block(e,0x41b647,0x41b67d);
    int initialized=(int)read(e,g+8,1),saved=(int)read(e,g+60,1),d48=(int)read(e,g+65,1);
    put(e,U+0x71,1,flags71);put(e,U+0x64,1,flags64);put(e,U+0x10,4,0);put(e,0x58aaaa,1,terrain);put(e,0x632194,4,g);e.writeRegister("ESI",U);block(e,0x452248,0x452273);int walk=(int)read(e,g+8,1);
    put(e,U+0x5e,1,role);e.writeRegister("EDI",U);e.writeRegister("ECX",0);block(e,0x44f229,0x44f249);int terr=(int)read(e,g+8,1);
    put(e,U+0x5f,1,120);e.writeRegister("EDI",U);e.setBreakpoint(toAddr(0x43e404));e.setBreakpoint(toAddr(0x43e44f));if(!e.run(toAddr(0x43e3ef),null,monitor))throw new Exception(e.getLastError());e.clearBreakpoint(toAddr(0x43e404));e.clearBreakpoint(toAddr(0x43e44f));int success=(int)read(e,g+8,1);
    e.writeRegister("ESI",U);block(e,0x43e430,0x43e44f);int failure=(int)read(e,g+8,1);
    out.println("{\"scenario\":"+s+",\"initialized\":"+initialized+",\"saved\":"+saved+",\"descriptor48\":"+d48+",\"walk\":"+walk+",\"terrain\":"+terr+",\"success\":"+success+",\"failure\":"+failure+"}");
   }finally{e.dispose();}
  }
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{for(int i=0;i<20;i++)out.println("{\"slot\":"+(i+1)+",\"x\":"+(byte)read(e,0x49add8+i*2,1)+",\"y\":"+(byte)read(e,0x49add8+i*2+1,1)+"}");}finally{e.dispose();}
 }}
}
