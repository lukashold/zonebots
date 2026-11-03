// Native idle world-order boundaries:00458620 raw group order copy and
// 0044ca50 withdrawal. Route creation is intercepted; no world simulation.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeIdleOrdersOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,G=U+0x1000,T=0x5e30c8+7*0xc0;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 void invoke(EmulatorHelper e,long a)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int role:new int[]{-1,0,1})for(int command:new int[]{0,12})for(int locked:new int[]{0,128})for(int group:new int[]{0,1,12})for(int coordinate:new int[]{-1,0,32767}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x2000]);put(e,0x632194,4,G);put(e,U+0x5e,1,role);put(e,U+0x5c,1,command);put(e,U+0x69,1,locked);put(e,U+0x50,4,88);put(e,U+0x54,2,99);put(e,U+0x56,2,111);put(e,G+0x3a,1,group);put(e,G+0x2c,2,coordinate);put(e,G+0x2e,2,123);put(e,G+0x30,2,456);e.setBreakpoint(toAddr(RET));invoke(e,0x458620);
    out.println("{\"kind\":\"sync\",\"role\":"+role+",\"command\":"+command+",\"locked\":"+locked+",\"group\":"+group+",\"coordinate\":"+coordinate+",\"nextCommand\":"+(byte)read(e,U+0x5c,1)+",\"f50\":"+(int)read(e,U+0x50,4)+",\"f54\":"+(short)read(e,U+0x54,2)+",\"f56\":"+(short)read(e,U+0x56,2)+"}");
   }finally{e.dispose();}
  }
  int[][] positions={{500,600},{1,1},{-1,-1},{32760,-32760}},targets={{600,700},{499,599},{-120,90}};
  for(int role:new int[]{-1,0,1})for(int[]p:positions)for(int[]t:targets){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x2000]);put(e,0x632194,4,G);put(e,G+0x3b,1,7);put(e,U+0x5e,1,role);put(e,U+0x2c,2,p[0]);put(e,U+0x2e,2,p[1]);put(e,T+0x2c,2,t[0]);put(e,T+0x2e,2,t[1]);for(long a:new long[]{RET,0x459c60,0x452810})e.setBreakpoint(toAddr(a));e.writeRegister("EDX",7);invoke(e,0x44ca50);int routed=0,x=0,y=0;
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();if(a==0x459c60){routed=1;x=(short)e.readRegister("EDX").intValue();y=(short)read(e,e.readRegister("ESP").longValue()+4,4);}else if(a==0x452810)e.writeRegister("EAX",0);else throw new Exception("Unexpected idle order boundary");resume(e);}
    out.println("{\"kind\":\"withdraw\",\"role\":"+role+",\"x\":"+p[0]+",\"y\":"+p[1]+",\"tx\":"+t[0]+",\"ty\":"+t[1]+",\"routed\":"+routed+",\"goalX\":"+x+",\"goalY\":"+y+",\"groupTarget\":"+read(e,G+0x3b,1)+",\"control8c\":"+read(e,U+0x8c,1)+"}");
   }finally{e.dispose();}
  }
 }}
}
