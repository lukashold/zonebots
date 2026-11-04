// Field-address inventory only; callers must establish the owning structure.
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import java.io.*;
public class FindNativeMask98Writers extends GhidraScript{
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  InstructionIterator it=currentProgram.getListing().getInstructions(true);
  while(it.hasNext()){Instruction i=it.next();String op=i.getDefaultOperandRepresentation(0);String m=i.getMnemonicString();
   if(op.contains(" + 0x98]")&&(m.equals("MOV")||m.equals("AND")||m.equals("OR")||m.equals("XOR")||m.equals("INC")||m.equals("DEC"))){Function f=currentProgram.getFunctionManager().getFunctionContaining(i.getAddress());out.println(i.getAddress()+" "+i+" FUNCTION "+(f==null?"null":f.getEntryPoint()+" "+f.getName()));}
  }
 }}
}
