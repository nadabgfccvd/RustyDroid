package com.example.roomfx;

import android.content.Context;
import androidx.room.ColumnInfo;
import androidx.room.Dao;
import androidx.room.Database;
import androidx.room.Entity;
import androidx.room.PrimaryKey;
import androidx.room.Query;
import androidx.room.Room;
import androidx.room.RoomDatabase;
import androidx.room.migration.Migration;
import androidx.sqlite.db.SupportSQLiteDatabase;

/** Esquema da versão 2: acrescenta `qty` (NOT NULL, default 0). */
@Entity(tableName = "item")
class ItemV2 {
    @PrimaryKey
    public int id;
    @ColumnInfo(name = "name")
    public String name;
    @ColumnInfo(name = "qty", defaultValue = "0")
    public int qty;
}

@Dao
interface ItemV2Dao {
    @Query("SELECT COUNT(*) FROM item")
    int count();

    @Query("SELECT qty FROM item WHERE id = 1")
    int qty();
}

@Database(entities = {ItemV2.class}, version = 2, exportSchema = false)
abstract class AppDbV2 extends RoomDatabase {
    abstract ItemV2Dao items();
}

/** Driver da versão 2: abre o banco da v1 com e sem a migração 1→2. */
public class V2 {
    static final Migration M1_2 = new Migration(1, 2) {
        @Override
        public void migrate(SupportSQLiteDatabase db) {
            db.execSQL("ALTER TABLE item ADD COLUMN qty INTEGER NOT NULL DEFAULT 0");
        }
    };

    /** Com a migração declarada: deve preservar a linha e aplicar a 1→2. */
    public static String withMigration(Context ctx, String name) {
        AppDbV2 db = Room.databaseBuilder(ctx, AppDbV2.class, name)
                .addMigrations(M1_2)
                .allowMainThreadQueries()
                .build();
        try {
            int count = db.items().count();
            return "count=" + count + ",qty=" + db.items().qty();
        } catch (RuntimeException e) {
            return "error:" + e.getClass().getName() + ":" + e.getMessage();
        } finally {
            db.close();
        }
    }

    /** Sem a migração: o Room real recusa a abertura (requireMigration padrão). */
    public static String withoutMigration(Context ctx, String name) {
        AppDbV2 db = Room.databaseBuilder(ctx, AppDbV2.class, name)
                .allowMainThreadQueries()
                .build();
        try {
            return "count=" + db.items().count();
        } catch (RuntimeException e) {
            return "error:" + e.getClass().getName() + ":" + e.getMessage();
        } finally {
            db.close();
        }
    }
}
